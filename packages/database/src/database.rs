use crate::error::Error;
use std::{ops::AsyncFnOnce, sync::Arc};

use crate::{
    infrastructure::database::database::Database as Backend,
    repositories::{
        job::JobRepository, repository::RepositoryRepository, secret::SecretRepository,
        stack::StackRepository, user::UserRepository,
    },
};

pub struct Database {
    pub repositories: Box<dyn RepositoryRepository>,
    pub stacks: Box<dyn StackRepository>,
    pub secrets: Box<dyn SecretRepository>,
    pub jobs: Box<dyn JobRepository>,
    pub users: Box<dyn UserRepository>,
    pub(crate) database: Arc<Backend>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, Error> {
        Ok(crate::infrastructure::connect(database_url).await?)
    }

    pub async fn transaction<T>(
        &self,
        callback: impl AsyncFnOnce() -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.database.transaction(callback).await?
    }
}
