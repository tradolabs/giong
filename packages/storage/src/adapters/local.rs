use crate::adapters::base::StorageAdapter;
use crate::error::Error;
use std::path::{Component, Path, PathBuf};

pub(crate) struct Local {
    root: PathBuf,
}

impl Local {
    pub(crate) fn new(storage_url: &str) -> Result<Self, Error> {
        let root = storage_url
            .strip_prefix("local://")
            .filter(|path| !path.is_empty())
            .ok_or_else(|| Error::Configuration("local storage path is required".into()))?;
        Ok(Self { root: PathBuf::from(root) })
    }

    fn path(&self, key: &str) -> Result<PathBuf, Error> {
        let path = Path::new(key);
        if key.is_empty()
            || !path.components().all(|component| matches!(component, Component::Normal(_)))
        {
            return Err(Error::BadRequest("invalid local storage key".into()));
        }
        Ok(self.root.join(path))
    }
}

#[async_trait::async_trait]
impl StorageAdapter for Local {
    async fn upload(&self, path: &Path, key: &str, _content_type: &str) -> Result<(), Error> {
        let destination = self.path(key)?;
        if let Some(parent) = destination.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::copy(path, destination).await?;
        Ok(())
    }

    async fn download(&self, key: &str) -> Result<Option<Vec<u8>>, Error> {
        match tokio::fs::read(self.path(key)?).await {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }
}
