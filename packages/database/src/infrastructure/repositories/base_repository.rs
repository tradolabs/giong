use anyhow::{Context, Result, anyhow, ensure};
use std::{marker::PhantomData, rc::Rc, str::FromStr};
use uuid::Uuid;

use crate::{
    Condition, Document, FieldValue, Operator, infrastructure::database::Database,
    repositories::base_repository::BaseRepository,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ColumnType, ConnectionTrait, DbBackend, EntityTrait,
    FromQueryResult, IdenStatic, Iterable, PrimaryKeyToColumn, QueryFilter, QueryOrder,
    QuerySelect, QueryTrait, Select, Statement, UpdateMany,
    sea_query::{Expr, Query, SqliteQueryBuilder},
};

pub(crate) struct SeaOrmRepository<E: EntityTrait> {
    database: Rc<Database>,
    entity: PhantomData<E>,
}

impl<E: EntityTrait> SeaOrmRepository<E> {
    pub(crate) fn new(database: Rc<Database>) -> Self {
        Self { database, entity: PhantomData }
    }

    pub(crate) fn connection(&self) -> impl ConnectionTrait + '_ {
        self.database.connection()
    }
}

#[async_trait::async_trait(?Send)]
impl<E, T> BaseRepository<T> for SeaOrmRepository<E>
where
    E: EntityTrait,
    E::ActiveModel: Send,
    E::Model: FromQueryResult + Sync,
    T: From<E::Model>,
{
    async fn create_one(&self, document: Document) -> Result<T> {
        let mut model = active_model::<E>(document)?;
        model.try_set(
            E::Column::from_str("id").map_err(|_| anyhow!("Entity has no ID column"))?,
            Uuid::now_v7().into(),
        )?;
        let mut query = E::insert(model).into_query();
        query.returning(Query::returning().all());
        let (sql, values) = query.build(SqliteQueryBuilder);
        let statement = Statement::from_sql_and_values(DbBackend::Sqlite, sql, values.0);
        let model = E::Model::find_by_statement(statement)
            .one(&self.connection())
            .await?
            .context("Insert returned no record")?;
        Ok(model.into())
    }

    async fn update_one(&self, condition: Condition, document: Document) -> Result<()> {
        ensure!(
            update::<E>(condition, document, Some(1))?
                .exec(&self.connection())
                .await?
                .rows_affected
                == 1,
            "No record matches the condition"
        );
        Ok(())
    }

    async fn update_many(&self, condition: Condition, document: Document) -> Result<u64> {
        Ok(update::<E>(condition, document, None)?
            .exec(&self.connection())
            .await?
            .rows_affected)
    }

    async fn find(&self, condition: Condition) -> Result<Vec<T>> {
        Ok(select::<E>(condition)?
            .all(&self.connection())
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    async fn find_one(&self, condition: Condition) -> Result<Option<T>> {
        Ok(select::<E>(condition)?
            .one(&self.connection())
            .await?
            .map(Into::into))
    }
}

fn update<E: EntityTrait>(
    condition: Condition,
    document: Document,
    limit: Option<u64>,
) -> Result<UpdateMany<E>> {
    let id = E::Column::from_str("id").map_err(|_| anyhow!("Entity has no ID column"))?;
    Ok(E::update_many()
        .set(active_model::<E>(document)?)
        .col_expr(
            E::Column::from_str("updated_at")
                .map_err(|_| anyhow!("Entity has no updated_at column"))?,
            Expr::cust("CURRENT_TIMESTAMP"),
        )
        .filter(
            id.in_subquery(
                select::<E>(condition)?
                    .select_only()
                    .column(id)
                    .limit(limit)
                    .into_query(),
            ),
        ))
}

fn active_model<E: EntityTrait>(document: Document) -> Result<E::ActiveModel> {
    let mut model = E::ActiveModel::default();
    for (field, value) in document.fields {
        let column = E::Column::from_str(&field).map_err(|_| anyhow!("Unknown field: {field}"))?;
        ensure!(
            !matches!(column.as_str(), "id" | "created_at" | "updated_at"),
            "Field {field} is generated automatically"
        );
        let value = match value {
            FieldValue::String(value) => value.into(),
            FieldValue::Uuid(value) => value.into(),
            FieldValue::Integer(value) => match column.def().get_column_type() {
                ColumnType::Integer => i32::try_from(value)?.into(),
                _ => value.into(),
            },
            FieldValue::Boolean(value) => value.into(),
            FieldValue::Null => match column.def().get_column_type() {
                ColumnType::Integer => sea_orm::Value::Int(None),
                ColumnType::BigInteger => sea_orm::Value::BigInt(None),
                ColumnType::Uuid => sea_orm::Value::Uuid(None),
                ColumnType::Boolean => sea_orm::Value::Bool(None),
                _ => sea_orm::Value::String(None),
            },
        };
        model
            .try_set(column, value)
            .with_context(|| format!("Invalid value for field {field}"))?;
    }
    Ok(model)
}

fn select<E: EntityTrait>(condition: Condition) -> Result<Select<E>> {
    let mut query = E::find();
    for (field, operator) in condition.filters {
        let column = E::Column::from_str(&field).map_err(|_| anyhow!("Unknown field: {field}"))?;
        query = query.filter(match operator {
            Operator::Eq(value) => match value {
                FieldValue::String(value) => column.eq(value),
                FieldValue::Uuid(value) => column.eq(value),
                FieldValue::Integer(value) => column.eq(value),
                FieldValue::Boolean(value) => column.eq(value),
                FieldValue::Null => column.is_null(),
            },
            Operator::In(values) => {
                let values: Vec<sea_orm::Value> = values
                    .into_iter()
                    .map(|value| match value {
                        FieldValue::String(value) => value.into(),
                        FieldValue::Uuid(value) => value.into(),
                        FieldValue::Integer(value) => value.into(),
                        FieldValue::Boolean(value) => value.into(),
                        FieldValue::Null => sea_orm::Value::String(None),
                    })
                    .collect();
                column.is_in(values)
            }
        });
    }
    for key in E::PrimaryKey::iter() {
        query = query.order_by_asc(key.into_column());
    }
    Ok(query)
}
