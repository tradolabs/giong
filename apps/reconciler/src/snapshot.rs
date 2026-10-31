use giong_provider::protocol::Resource;
use giong_storage::storage::Storage;
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Default, Deserialize, Serialize)]
#[serde(transparent)]
pub(crate) struct Snapshot {
    resources: Vec<Resource>,
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
            package: resource.package.clone(),
            provider: resource.provider.clone(),
            inputs: resource.inputs.clone(),
        };
        if let Some(existing) = self.resources.iter_mut().find(|existing| {
            existing.package == resource.package
                && existing.name == resource.name
                && existing.provider == resource.provider
        }) {
            *existing = snapshot;
        } else {
            self.resources.push(snapshot);
        }
    }
}
