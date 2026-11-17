use crate::error::Error;
use crate::repositories::node::NodeRepository;
use crate::repositories::{secret::SecretRepository, vault::VaultRepository};
use std::{ops::AsyncFnOnce, sync::Arc};

use crate::{
    infrastructure::database::database::Database as Backend,
    repositories::{flow::FlowRepository, user::UserRepository},
};

pub struct Database {
    pub nodes: Box<dyn NodeRepository>,
    pub vaults: Box<dyn VaultRepository>,
    pub secrets: Box<dyn SecretRepository>,
    pub flows: Box<dyn FlowRepository>,
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
