mod adapters;

use anyhow::Result;
use std::path::Path;

#[async_trait::async_trait]
pub trait StorageAdapter: Send + Sync {
    async fn upload(&self, path: &Path, key: &str, content_type: &str) -> Result<()>;
}

pub struct Storage {
    adapter: Box<dyn StorageAdapter>,
}

impl Storage {
    pub fn new(storage_url: &str) -> Result<Self> {
        let adapter: Box<dyn StorageAdapter> = if storage_url.starts_with("s3:") {
            Box::new(adapters::s3::S3::new(storage_url)?)
        } else {
            anyhow::bail!("Unsupported storage URL scheme; use s3:")
        };

        Ok(Self { adapter })
    }

    pub async fn upload(&self, path: &Path, key: &str, content_type: &str) -> Result<()> {
        self.adapter.upload(path, key, content_type).await
    }
}
