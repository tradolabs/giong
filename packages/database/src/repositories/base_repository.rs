use anyhow::Result;

use super::types::{Condition, Document};

#[async_trait::async_trait(?Send)]
pub trait BaseRepository<T> {
    async fn create_one(&self, document: Document) -> Result<T>;
    async fn update_one(&self, condition: Condition, document: Document) -> Result<()>;
    async fn update_many(&self, condition: Condition, document: Document) -> Result<u64>;
    async fn find(&self, condition: Condition) -> Result<Vec<T>>;
    async fn find_one(&self, condition: Condition) -> Result<Option<T>>;
    async fn exists(&self, condition: Condition) -> Result<bool> {
        Ok(self.find_one(condition).await?.is_some())
    }
}
