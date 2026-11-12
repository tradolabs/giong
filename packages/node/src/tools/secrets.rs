use crate::runtime::{Host, bindings};
use std::sync::Arc;

#[derive(Clone)]
pub struct Secrets {
    pub get: Arc<dyn Fn(&str) -> anyhow::Result<String> + Send + Sync>,
}
impl bindings::giong::node::secrets::Host for Host {
    fn get(&mut self, key: String) -> Result<String, String> {
        let secrets = self.secrets.as_ref().ok_or("Secret access is not granted")?;
        (secrets.get)(&key).map_err(|_| "Secret lookup failed".into())
    }
}
