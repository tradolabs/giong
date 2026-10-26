use migrations::{Migrator, MigratorTrait};
use sea_orm::{DatabaseConnection, DbErr, TransactionTrait};
use std::{ops::AsyncFnOnce, sync::Arc};

use super::super::repositories::{
    job::SeaOrmJobRepository, repository::SeaOrmRepositoryRepository, stack::SeaOrmStackRepository,
    user::SeaOrmUserRepository,
};
use super::{connection::Connection, transaction_context};

pub(crate) enum Database {
    Sqlite(DatabaseConnection),
    D1(DatabaseConnection),
}

impl Database {
    pub(crate) async fn open(database_url: &str) -> Result<Self, DbErr> {
        if database_url.starts_with("d1:") {
            let database = super::variants::connect_d1(database_url).await?;
            Migrator::up(&database, None).await?;
            Ok(Self::D1(database))
        } else if database_url.starts_with("sqlite:") {
            let database = super::variants::connect_sqlite(database_url).await?;
            Migrator::up(&database, None).await?;
            Ok(Self::Sqlite(database))
        } else {
            Err(DbErr::Custom("unsupported database URL scheme".into()))
        }
    }

    pub(crate) fn connection(self: &Arc<Self>) -> Connection<'_> {
        let database = match self.as_ref() {
            Self::Sqlite(database) | Self::D1(database) => database,
        };
        match transaction_context::current(Arc::as_ptr(self) as usize) {
            Some(transaction) => Connection::Transaction(transaction),
            None => Connection::Database(database),
        }
    }

    pub(crate) fn into_database(self: Arc<Self>) -> crate::database::Database {
        crate::database::Database {
            repositories: Box::new(SeaOrmRepositoryRepository::new(self.clone())),
            stacks: Box::new(SeaOrmStackRepository::new(self.clone())),
            jobs: Box::new(SeaOrmJobRepository::new(self.clone())),
            users: Box::new(SeaOrmUserRepository::new(self.clone())),
            database: self,
        }
    }

    pub(crate) async fn transaction<T, E>(
        self: &Arc<Self>,
        callback: impl AsyncFnOnce() -> Result<T, E>,
    ) -> Result<Result<T, E>, DbErr> {
        if matches!(**self, Self::D1(_)) {
            return Ok(callback().await);
        }

        let tx = Arc::new(self.connection().executor().begin().await?);
        let result =
            transaction_context::scope(Arc::as_ptr(self) as usize, tx.clone(), callback()).await;
        let tx =
            Arc::try_unwrap(tx).map_err(|_| DbErr::Custom("Transaction is still in use".into()))?;

        if result.is_ok() {
            tx.commit().await?;
        } else {
            tx.rollback().await?;
        }

        Ok(result)
    }
}
