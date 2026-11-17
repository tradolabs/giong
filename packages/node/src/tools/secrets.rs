use crate::runtime::{Host, bindings};
use std::sync::Arc;

#[derive(Clone)]
pub struct Secrets {
    pub get: Arc<dyn Fn(Option<&str>, &str) -> anyhow::Result<String> + Send + Sync>,
}
impl bindings::giong::node::secrets::Host for Host {
    fn get(&mut self, key: String) -> Result<String, String> {
        let secrets = self.secrets.as_ref().ok_or("Secret access is not granted")?;
        (secrets.get)(None, &key).map_err(|_| "Secret lookup failed in flow vault".into())
    }

    fn get_from_vault(&mut self, vault: String, key: String) -> Result<String, String> {
        let secrets = self.secrets.as_ref().ok_or("Secret access is not granted")?;
        (secrets.get)(Some(&vault), &key).map_err(|_| {
            "Secret lookup failed: check the vault reference and flow attachment".into()
        })
    }
}
