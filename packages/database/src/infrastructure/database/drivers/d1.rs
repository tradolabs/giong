use anyhow::{Context, Result, ensure};
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
    pub(crate) fn new(database_url: &str) -> Result<Self> {
        let url = Url::parse(database_url)?;
        let account = url.host_str().context("Missing D1 account ID")?;
        let database = Uuid::parse_str(url.path().trim_start_matches('/'))?;
        let token = url
            .password()
            .filter(|v| !v.is_empty())
            .context("Missing D1 API token")?
            .to_owned();

        ensure!(
            account.len() == 32 && account.bytes().all(|b| b.is_ascii_hexdigit()),
            "Invalid D1 account ID"
        );
        ensure!(
            url.username().is_empty() && url.query().is_none() && url.fragment().is_none(),
            "Invalid D1 URL"
        );

        Ok(Self {
            client: Client::builder().timeout(Duration::from_secs(30)).build()?,
            endpoint: format!(
                "https://api.cloudflare.com/client/v4/accounts/{account}/d1/database/{database}/query"
            ),
            token,
        })
    }

    async fn request(&self, statement: Statement) -> Result<QueryResult> {
        let params = statement
            .values
            .into_iter()
            .flat_map(|v| v.0)
            .map(Self::bind_parameter)
            .collect::<Result<Vec<_>>>()?;

        let response: Response = self
            .client
            .post(&self.endpoint)
            .bearer_auth(&self.token)
            .json(&json!({
                "sql": statement.sql,
                "params": params,
            }))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        ensure!(response.success, "D1 query failed");

        let result = response
            .result
            .into_iter()
            .next()
            .context("Empty D1 result")?;

        ensure!(result.success, "D1 query failed");

        Ok(result)
    }

    fn bind_row(row: Json) -> Result<ProxyRow> {
        let Json::Object(row) = row else {
            anyhow::bail!("Invalid D1 row");
        };

        let values = row
            .into_iter()
            .map(|(name, json)| Ok((name, Self::bind_response(json)?)))
            .collect::<Result<_>>()?;

        Ok(ProxyRow { values })
    }

    fn bind_response(value: Json) -> Result<Value> {
        Ok(match value {
            Json::Null => Value::String(None),
            Json::Bool(value) => value.into(),
            Json::Number(value) if value.is_i64() => value.as_i64().unwrap().into(),
            Json::Number(value) => value.as_f64().context("Invalid number")?.into(),
            Json::String(value) => value.into(),
            _ => anyhow::bail!("Unsupported D1 response value"),
        })
    }

    fn bind_parameter(value: Value) -> Result<Json> {
        Ok(match value {
            Value::String(value) => json!(value),
            Value::Uuid(value) => json!(value),
            Value::Bool(value) => json!(value),
            Value::Int(value) => json!(value),
            Value::BigInt(value) => json!(value),
            Value::Float(value) => json!(value),
            Value::Double(value) => json!(value),
            _ => anyhow::bail!("Unsupported D1 parameter type"),
        })
    }
}

#[async_trait::async_trait]
impl ProxyDatabaseTrait for D1 {
    async fn query(&self, statement: Statement) -> Result<Vec<ProxyRow>, DbErr> {
        let query_result = self
            .request(statement)
            .await
            .map_err(|error| DbErr::Custom(error.to_string()))?;
        query_result
            .values
            .into_iter()
            .map(|row| Self::bind_row(row).map_err(|error| DbErr::Custom(error.to_string())))
            .collect()
    }

    async fn execute(&self, statement: Statement) -> Result<ProxyExecResult, DbErr> {
        let query_result = self
            .request(statement)
            .await
            .map_err(|error| DbErr::Custom(error.to_string()))?;

        Ok(ProxyExecResult::new(
            query_result.meta.last_row_id,
            query_result.meta.changes,
        ))
    }
}
