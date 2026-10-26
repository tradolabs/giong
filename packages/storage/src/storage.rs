use crate::adapters::base::StorageAdapter;
use crate::adapters::local::Local;
use crate::adapters::s3::S3;
use crate::error::Error;
use std::path::Path;

pub struct Storage {
    adapter: Box<dyn StorageAdapter>,
}

impl Storage {
    pub fn new(storage_url: &str) -> Result<Self, Error> {
        let adapter: Box<dyn StorageAdapter> = if storage_url.starts_with("s3:") {
            Box::new(S3::new(storage_url)?)
        } else if storage_url.starts_with("local://") {
            Box::new(Local::new(storage_url)?)
        } else {
            return Err(Error::Configuration("unsupported storage URL scheme".to_string()));
        };

        Ok(Self { adapter })
    }

    pub async fn upload(&self, path: &Path, key: &str, content_type: &str) -> Result<(), Error> {
        self.adapter.upload(path, key, content_type).await
    }

    pub async fn download(&self, key: &str) -> Result<Option<Vec<u8>>, Error> {
        self.adapter.download(key).await
    }
}
