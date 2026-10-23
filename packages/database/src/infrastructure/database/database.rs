use anyhow::{Result, anyhow};
use migrations::{Migrator, MigratorTrait};
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::{ops::AsyncFnOnce, rc::Rc, sync::Arc};

use super::super::repositories::{
    job_repository::SeaOrmJobRepository, repository_repository::SeaOrmRepository,
    stack_repository::SeaOrmStackRepository,
};
use super::{connection::Connection, transaction_context};

pub(crate) enum Database {
    Sqlite(DatabaseConnection),
    D1(DatabaseConnection),
}

impl Database {
    pub(crate) async fn open(database_url: &str) -> Result<Self> {
        if database_url.starts_with("d1:") {
            let database = super::variants::connect_d1(database_url).await?;
            Migrator::up(&database, None).await?;
            Ok(Self::D1(database))
        } else if database_url.starts_with("sqlite:") {
            let database = super::variants::connect_sqlite(database_url).await?;
            Migrator::up(&database, None).await?;
            Ok(Self::Sqlite(database))
        } else {
            anyhow::bail!("Unsupported database URL scheme; use sqlite: or d1:")
        }
    }

    pub(crate) fn connection(self: &Rc<Self>) -> Connection<'_> {
        let database = match self.as_ref() {
            Self::Sqlite(database) | Self::D1(database) => database,
        };
        match transaction_context::current(Rc::as_ptr(self) as usize) {
            Some(transaction) => Connection::Transaction(transaction),
            None => Connection::Database(database),
        }
    }

    pub(crate) fn into_database(self: Rc<Self>) -> crate::Database {
        crate::Database {
            repositories: Box::new(SeaOrmRepository::new(self.clone())),
            stacks: Box::new(SeaOrmStackRepository::new(self.clone())),
            jobs: Box::new(SeaOrmJobRepository::new(self.clone())),
            database: self,
        }
    }

    pub(crate) async fn transaction<T>(
        self: &Rc<Self>,
        callback: impl AsyncFnOnce() -> Result<T>,
    ) -> Result<T> {
        if matches!(**self, Self::D1(_)) {
            return callback().await;
        }

        let tx = Arc::new(self.connection().executor().begin().await?);
        let result =
            transaction_context::scope(Rc::as_ptr(self) as usize, tx.clone(), callback()).await;
        let tx = Arc::try_unwrap(tx).map_err(|_| anyhow!("Transaction is still in use"))?;

        if result.is_ok() {
            tx.commit().await?;
        } else {
            tx.rollback().await?;
        }

        result
    }
}
