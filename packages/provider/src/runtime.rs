use crate::error::Error;
use crate::oci;
use crate::protocol::ChangeStatus;
use crate::wasm::exports::giong::provider::lifecycle::ChangeStatus as ProviderChangeStatus;
use crate::wasm::{self, Host, Provider};
use serde_json::Value;
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use wasmtime::component::{Component, Linker};
use wasmtime::{Engine, Store};

#[derive(Clone)]
pub struct Runtime {
    engine: Engine,
    linker: Linker<Host>,
    directory: PathBuf,
    registry: String,
    version: String,
    components: Arc<Mutex<HashMap<String, Component>>>,
    client: reqwest::Client,
}

impl Runtime {
    pub fn new(directory: PathBuf, registry: &str, version: &str) -> Result<Self, Error> {
        let (engine, linker) = wasm::setup()?;
        Ok(Self {
            engine,
            linker,
            directory,
            registry: registry.into(),
            version: version.into(),
            components: Arc::new(Mutex::new(HashMap::new())),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .map_err(Error::unexpected)?,
        })
    }

    async fn instantiate(&self, resource_type: &str) -> Result<(Store<Host>, Provider), Error> {
        let component = {
            let mut components = self.components.lock().await;
            if let Some(component) = components.get(resource_type) {
                component.clone()
            } else {
                let component = oci::load(
                    &self.engine,
                    &self.directory,
                    &self.registry,
                    &self.version,
                    resource_type,
                )
                .await?;
                components.insert(resource_type.into(), component.clone());
                component
            }
        };
        let mut store = Store::new(&self.engine, Host::new(self.client.clone()));
        store.limiter(|host| &mut host.limits);
        store.set_fuel(100_000_000).map_err(Error::unexpected)?;
        let bindings = Provider::instantiate_async(&mut store, &component, &self.linker)
            .await
            .map_err(Error::unexpected)?;
        Ok((store, bindings))
    }

    pub async fn read(&self, resource_type: &str, config: &Value) -> Result<ChangeStatus, Error> {
        let (mut store, bindings) = self.instantiate(resource_type).await?;
        let status = bindings
            .giong_provider_lifecycle()
            .call_read(&mut store, &config.to_string())
            .await
            .map_err(Error::unexpected)?
            .map_err(Error::Provider)?;
        Ok(match status {
            ProviderChangeStatus::Created => ChangeStatus::Created,
            ProviderChangeStatus::Updated => ChangeStatus::Updated,
            ProviderChangeStatus::Unchanged => ChangeStatus::Unchanged,
        })
    }

    pub async fn apply(&self, resource_type: &str, config: &Value) -> Result<Value, Error> {
        let (mut store, bindings) = self.instantiate(resource_type).await?;
        let state = bindings
            .giong_provider_lifecycle()
            .call_apply(&mut store, &config.to_string())
            .await
            .map_err(Error::unexpected)?
            .map_err(Error::Provider)?;
        serde_json::from_str(&state).map_err(Error::unexpected)
    }
}
