use anyhow::Result;
use std::{ops::AsyncFnOnce, rc::Rc};

use crate::{
    infrastructure::database::Database as Backend,
    repositories::{
        job_repository::JobRepository, repository_repository::RepositoryRepository,
        stack_repository::StackRepository,
    },
};

pub struct Database {
    pub repositories: Box<dyn RepositoryRepository>,
    pub stacks: Box<dyn StackRepository>,
    pub jobs: Box<dyn JobRepository>,
    pub(crate) database: Rc<Backend>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        crate::infrastructure::connect(database_url).await
    }

    pub async fn transaction<T>(&self, callback: impl AsyncFnOnce() -> Result<T>) -> Result<T> {
        self.database.transaction(callback).await
    }
}
