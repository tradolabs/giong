use crate::runtime::{Host, bindings};
use std::sync::Arc;
#[derive(Clone)]
pub struct Events {
    pub emit: Arc<dyn Fn(&str) -> anyhow::Result<()> + Send + Sync>,
}
impl bindings::giong::node::events::Host for Host {
    fn emit(&mut self, event: String) -> Result<(), String> {
        let events = self.events.as_ref().ok_or("Event access is not granted")?;
        (events.emit)(&event).map_err(|_| "Could not publish node event".into())
    }
}
