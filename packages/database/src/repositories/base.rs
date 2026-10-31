use super::types::{condition::Condition, document::Document};
use crate::error::Error;

#[async_trait::async_trait]
pub trait Repository<T>: Send + Sync {
    async fn create_one(&self, document: Document) -> Result<T, Error>;
    async fn update_one(&self, condition: Condition, document: Document) -> Result<bool, Error>;
    async fn update_many(&self, condition: Condition, document: Document) -> Result<u64, Error>;
    async fn delete_one(&self, condition: Condition) -> Result<bool, Error>;
    async fn find(&self, condition: Condition) -> Result<Vec<T>, Error>;
    async fn find_one(&self, condition: Condition) -> Result<Option<T>, Error>;
    async fn exists(&self, condition: Condition) -> Result<bool, Error> {
        Ok(self.find_one(condition).await?.is_some())
    }
}
