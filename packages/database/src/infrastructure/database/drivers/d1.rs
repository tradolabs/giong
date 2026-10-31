use reqwest::{Client, Url};
use sea_orm::{DbErr, ProxyDatabaseTrait, ProxyExecResult, ProxyRow, Statement, Value};
use serde::Deserialize;
use serde_json::{Value as Json, json};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug)]
pub(crate) struct D1 {
    client: Client,
    endpoint: String,
    token: String,
}

#[derive(Deserialize)]
struct Response {
    success: bool,
    #[serde(default)]
    errors: Vec<Json>,
    #[serde(default)]
    result: Vec<QueryResult>,
}

#[derive(Deserialize)]
struct QueryResult {
    success: bool,
    #[serde(default, rename = "results")]
    values: Vec<Json>,
    meta: Meta,
}

#[derive(Deserialize)]
struct Meta {
    #[serde(default)]
    changes: u64,
    #[serde(default)]
    last_row_id: u64,
}

impl D1 {
    pub(crate) fn new(database_url: &str) -> Result<Self, DbErr> {
        let database_url =
            Url::parse(database_url).map_err(|_| DbErr::Custom("invalid D1 URL".into()))?;

        let account = database_url
            .host_str()
            .ok_or(DbErr::Custom("missing or invalid D1 account ID".into()))?;
        let database = Uuid::parse_str(database_url.path().trim_start_matches('/'))
            .map_err(|_| DbErr::Custom("missing or invalid D1 database ID".into()))?;
        let token = database_url
            .password()
            .filter(|v| !v.is_empty())
            .ok_or(DbErr::Custom("missing D1 API token".into()))?
            .to_owned();

        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .map_err(|error| DbErr::Custom(error.to_string()))?,
            endpoint: format!(
                "https://api.cloudflare.com/client/v4/accounts/{account}/d1/database/{database}/query"
            ),
            token,
        })
    }

    async fn request(&self, statement: Statement) -> Result<QueryResult, DbErr> {
        let params = statement
            .values
            .into_iter()
            .flat_map(|v| v.0)
            .map(Self::bind_parameter)
            .collect::<Result<Vec<_>, DbErr>>()?;

        let response = self
            .client
            .post(&self.endpoint)
            .bearer_auth(&self.token)
            .json(&json!({"sql": statement.sql,"params": params}))
            .send()
            .await
            .map_err(|error| DbErr::Custom(format!("D1 request failed: {error}")))?;
        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .map_err(|error| DbErr::Custom(format!("Could not read D1 error: {error}")))?;
            return Err(DbErr::Custom(format!("D1 returned HTTP {status}: {body}")));
        }
        let response: Response = response
            .json()
            .await
            .map_err(|error| DbErr::Custom(format!("invalid D1 response: {error}")))?;

        if !response.success {
            return Err(DbErr::Custom(format!("D1 query failed: {}", json!(response.errors))));
        }

        let result = response
            .result
            .into_iter()
            .next()
            .ok_or_else(|| DbErr::Custom("empty D1 result".into()))?;

        if !result.success {
            return Err(DbErr::Custom("d1 query failed".into()));
        }

        Ok(result)
    }

    fn bind_row(row: Json) -> Result<ProxyRow, DbErr> {
        let Json::Object(row) = row else {
            return Err(DbErr::Custom("invalid D1 row".into()));
        };

        let values = row
            .into_iter()
            .map(|(name, json)| Ok((name, Self::bind_response(json)?)))
            .collect::<Result<_, DbErr>>()?;

        Ok(ProxyRow { values })
    }

    fn bind_response(value: Json) -> Result<Value, DbErr> {
        Ok(match value {
            Json::Null => Value::String(None),
            Json::Bool(v) => v.into(),
            Json::Number(v) if v.is_i64() => v.as_i64().unwrap().into(),
            Json::Number(v) => {
                v.as_f64().ok_or_else(|| DbErr::Custom("invalid D1 number".into()))?.into()
            }
            Json::String(v) => Uuid::parse_str(&v).map_or_else(|_| v.into(), Into::into),
            _ => return Err(DbErr::Custom("unsupported D1 response value".into())),
        })
    }

    fn bind_parameter(value: Value) -> Result<Json, DbErr> {
        Ok(match value {
            Value::String(value) => json!(value),
            Value::Uuid(value) => json!(value),
            Value::Bool(value) => json!(value),
            Value::TinyInt(value) => json!(value),
            Value::SmallInt(value) => json!(value),
            Value::Int(value) => json!(value),
            Value::BigInt(value) => json!(value),
            Value::TinyUnsigned(value) => json!(value),
            Value::SmallUnsigned(value) => json!(value),
            Value::Unsigned(value) => json!(value),
            Value::BigUnsigned(value) => json!(value),
            Value::Float(value) => json!(value),
            Value::Double(value) => json!(value),
            _ => return Err(DbErr::Custom("unsupported D1 parameter type".into())),
        })
    }
}

#[async_trait::async_trait]
impl ProxyDatabaseTrait for D1 {
    async fn query(&self, statement: Statement) -> Result<Vec<ProxyRow>, DbErr> {
        let query_result = self.request(statement).await?;
        query_result.values.into_iter().map(Self::bind_row).collect()
    }

    async fn execute(&self, statement: Statement) -> Result<ProxyExecResult, DbErr> {
        let query_result = self.request(statement).await?;
        Ok(ProxyExecResult::new(query_result.meta.last_row_id, query_result.meta.changes))
    }
}
