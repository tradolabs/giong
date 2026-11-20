use anyhow::{Context, Result};
use giong_kv::error::Error as KvError;
use giong_kv::kv::Namespace;

#[derive(Clone)]
pub struct Storage {
    namespace: Namespace,
}

impl Storage {
    pub fn new(namespace: Namespace) -> Self {
        Self { namespace }
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        match self.namespace.get(key.as_bytes()) {
            Ok(value) => Ok(Some(value)),
            Err(KvError::NotFound) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn put(&self, key: &str, value: &[u8]) -> Result<()> {
        Ok(self.namespace.put(key.as_bytes(), value)?)
    }

    pub fn delete(&self, key: &str) -> Result<bool> {
        Ok(self.namespace.delete(key.as_bytes())?)
    }

    pub fn search(&self, prefix: &str) -> Result<Vec<String>> {
        self.namespace
            .search(prefix.as_bytes())?
            .into_iter()
            .map(|(key, _)| String::from_utf8(key).context("Storage key is not UTF-8"))
            .collect()
    }
}
