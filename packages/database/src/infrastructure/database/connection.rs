use sea_orm::{
    ConnectionTrait, DatabaseConnection, DatabaseExecutor, DatabaseTransaction, DbBackend, DbErr,
    ExecResult, QueryResult, Statement,
};
use std::sync::Arc;

pub(crate) enum Connection<'a> {
    Database(&'a DatabaseConnection),
    Transaction(Arc<DatabaseTransaction>),
}

impl Connection<'_> {
    pub(crate) fn executor(&self) -> DatabaseExecutor<'_> {
        match self {
            Self::Database(database) => DatabaseExecutor::Connection(database),
            Self::Transaction(transaction) => DatabaseExecutor::Transaction(transaction),
        }
    }
}

#[async_trait::async_trait]
impl ConnectionTrait for Connection<'_> {
    fn get_database_backend(&self) -> DbBackend {
        self.executor().get_database_backend()
    }

    async fn execute_raw(&self, statement: Statement) -> Result<ExecResult, DbErr> {
        self.executor().execute_raw(statement).await
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        self.executor().execute_unprepared(sql).await
    }

    async fn query_one_raw(&self, statement: Statement) -> Result<Option<QueryResult>, DbErr> {
        self.executor().query_one_raw(statement).await
    }

    async fn query_all_raw(&self, statement: Statement) -> Result<Vec<QueryResult>, DbErr> {
        self.executor().query_all_raw(statement).await
    }
}
