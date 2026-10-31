use giong_provider::protocol::Resource;
use giong_storage::storage::Storage;
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Default, Deserialize, Serialize)]
#[serde(transparent)]
pub(crate) struct Snapshot {
    resources: Vec<Resource>,
}

pub(crate) fn storage_key(stack_id: &str) -> Option<String> {
    if stack_id.is_empty()
        || !stack_id.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return None;
    }
    Some(format!("snapshots/{stack_id}"))
}

impl Snapshot {
    pub(crate) async fn load(storage: &Storage, key: &str) -> anyhow::Result<Self> {
        match storage.download(key).await? {
            Some(bytes) => Ok(serde_json::from_slice(&bytes)?),
            None => Ok(Self::default()),
        }
    }

    pub(crate) async fn save(&self, storage: &Storage, key: &str) -> anyhow::Result<()> {
        let mut file = tempfile::NamedTempFile::new()?;
        serde_json::to_writer(&mut file, self)?;
        file.flush()?;
        storage.upload(file.path(), key, "application/json").await?;
        Ok(())
    }

    pub(crate) fn insert(&mut self, resource: &Resource) {
        let snapshot = Resource {
            name: resource.name.clone(),
            resource_type: resource.resource_type.clone(),
            config: resource.config.clone(),
        };
        if let Some(existing) = self.resources.iter_mut().find(|existing| {
            existing.resource_type == resource.resource_type && existing.name == resource.name
        }) {
            *existing = snapshot;
        } else {
            self.resources.push(snapshot);
        }
    }
}
