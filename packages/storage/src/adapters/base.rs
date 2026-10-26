use crate::error::Error;
use std::path::Path;

#[async_trait::async_trait]
pub trait StorageAdapter: Send + Sync {
    async fn upload(&self, path: &Path, key: &str, content_type: &str) -> Result<(), Error>;
    async fn download(&self, key: &str) -> Result<Option<Vec<u8>>, Error>;
}
