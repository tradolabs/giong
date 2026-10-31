use crate::error::Error;
use std::{marker::PhantomData, str::FromStr, sync::Arc};
use uuid::Uuid;

use crate::infrastructure::database::database::Database;
use crate::repositories::base::Repository;
use crate::repositories::types::condition::{Condition, Operator};
use crate::repositories::types::document::Document;
use crate::repositories::types::field_value::FieldValue;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ColumnType, ConnectionTrait, DbBackend, EntityTrait,
    FromQueryResult, Iterable, PrimaryKeyToColumn, QueryFilter, QueryOrder, QuerySelect,
    QueryTrait, Select, Statement, UpdateMany,
    sea_query::{Expr, Query, SqliteQueryBuilder},
};

pub(crate) struct SeaOrmRepository<E: EntityTrait> {
    database: Arc<Database>,
    entity: PhantomData<E>,
}

impl<E: EntityTrait> SeaOrmRepository<E> {
    pub(crate) fn new(database: Arc<Database>) -> Self {
        Self { database, entity: PhantomData }
    }

    pub(crate) fn connection(&self) -> impl ConnectionTrait + '_ {
        self.database.connection()
    }
}

impl<E> SeaOrmRepository<E>
where
    E: EntityTrait,
{
    fn query_find(condition: Condition) -> Result<Select<E>, Error> {
        let mut query = E::find().filter(Self::to_condition(condition)?);
        for key in E::PrimaryKey::iter() {
            query = query.order_by_asc(key.into_column());
        }
        Ok(query)
    }

    fn query_update(condition: Condition, document: Document) -> Result<UpdateMany<E>, Error> {
        let model = Self::to_model(document)?;
        let condition = Self::to_condition(condition)?;
        let mut query = E::update_many().set(model).filter(condition);
        if let Ok(updated_column) = E::Column::from_str("updated_at") {
            query = query.col_expr(updated_column, Expr::cust("CURRENT_TIMESTAMP"));
        }
        Ok(query)
    }

    fn to_model(document: Document) -> Result<E::ActiveModel, Error> {
        let mut model = E::ActiveModel::default();

        for (field, value) in document.fields {
            let column = E::Column::from_str(&field)
                .map_err(|_| Error::Unexpected(format!("Unknown field: {field}")))?;

            let value = match value {
                FieldValue::Null => match column.def().get_column_type() {
                    ColumnType::Integer => sea_orm::Value::Int(None),
                    ColumnType::BigInteger => sea_orm::Value::BigInt(None),
                    ColumnType::Uuid => sea_orm::Value::Uuid(None),
                    ColumnType::Boolean => sea_orm::Value::Bool(None),
                    _ => sea_orm::Value::String(None),
                },
                value => Self::to_value(value),
            };
            model.try_set(column, value).map_err(|source| {
                Error::Unexpected(format!("Invalid value for field {field}: {source}"))
            })?;
        }

        Ok(model)
    }

    fn to_condition(condition: Condition) -> Result<sea_orm::Condition, Error> {
        let mut db_condition = sea_orm::Condition::all();

        for (field, operator) in condition.filters {
            let column = E::Column::from_str(&field)
                .map_err(|_| Error::Unexpected(format!("Unknown field: {field}")))?;
            let expression = match operator {
                Operator::Eq(FieldValue::Null) => column.is_null(),
                Operator::Eq(value) => column.eq(Self::to_value(value)),
                Operator::In(values) => {
                    let values: Vec<sea_orm::Value> =
                        values.into_iter().map(Self::to_value).collect();
                    column.is_in(values)
                }
            };
            db_condition = db_condition.add(expression);
        }

        Ok(db_condition)
    }

    fn to_value(value: FieldValue) -> sea_orm::Value {
        match value {
            FieldValue::String(value) => value.into(),
            FieldValue::Uuid(value) => value.into(),
            FieldValue::Integer(value) => value.into(),
            FieldValue::Boolean(value) => value.into(),
            FieldValue::Null => sea_orm::Value::String(None),
        }
    }
}

#[async_trait::async_trait]
impl<E, T> Repository<T> for SeaOrmRepository<E>
where
    E: EntityTrait,
    E::ActiveModel: Send,
    E::Model: FromQueryResult + Sync,
    T: From<E::Model>,
{
    async fn create_one(&self, document: Document) -> Result<T, Error> {
        let has_id_value = document.fields.contains_key("id");
        let mut model = Self::to_model(document)?;

        if !has_id_value && let Ok(column) = E::Column::from_str("id") {
            model.try_set(column, Uuid::now_v7().into())?;
        }

        let mut query = E::insert(model).into_query();
        query.returning(Query::returning().all());
        let (sql, values) = query.build(SqliteQueryBuilder);
        let statement = Statement::from_sql_and_values(DbBackend::Sqlite, sql, values.0);
        let model = E::Model::find_by_statement(statement)
            .one(&self.connection())
            .await?
            .ok_or(Error::Unexpected("insert returned no record".into()))?;

        Ok(model.into())
    }

    async fn update_one(&self, condition: Condition, document: Document) -> Result<bool, Error> {
        let result = {
            E::PrimaryKey::iter().try_fold(
                Self::query_update(condition.clone(), document)?,
                |query, key| -> Result<_, Error> {
                    let column = key.into_column();
                    let first = Self::query_find(condition.clone())?
                        .select_only()
                        .column(column)
                        .limit(1)
                        .into_query();
                    Ok(query.filter(column.in_subquery(first)))
                },
            )?
        }
        .exec(&self.connection())
        .await?;
        Ok(result.rows_affected == 1)
    }

    async fn update_many(&self, condition: Condition, document: Document) -> Result<u64, Error> {
        let result = Self::query_update(condition, document)?.exec(&self.connection()).await?;
        Ok(result.rows_affected)
    }

    async fn delete_one(&self, condition: Condition) -> Result<bool, Error> {
        let mut query = E::delete_many();
        for key in E::PrimaryKey::iter() {
            let column = key.into_column();
            let first = Self::query_find(condition.clone())?
                .select_only()
                .column(column)
                .limit(1)
                .into_query();
            query = query.filter(column.in_subquery(first));
        }
        let result = query.exec(&self.connection()).await?;
        Ok(result.rows_affected == 1)
    }

    async fn find(&self, condition: Condition) -> Result<Vec<T>, Error> {
        let result = Self::query_find(condition)?.all(&self.connection()).await?;
        Ok(result.into_iter().map(Into::into).collect())
    }

    async fn find_one(&self, condition: Condition) -> Result<Option<T>, Error> {
        let result = Self::query_find(condition)?.one(&self.connection()).await?;
        Ok(result.map(Into::into))
    }
}
