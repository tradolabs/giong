use crate::runtime::{Host, bindings};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

pub type Outputs = Arc<Mutex<BTreeMap<String, Value>>>;

impl bindings::giong::node::nodes::Host for Host {
    fn scope(&mut self) -> Result<String, String> {
        self.scope.clone().ok_or_else(|| "Flow scope is not available".into())
    }

    fn output(&mut self, name: String) -> Result<String, String> {
        let outputs = self.outputs.as_ref().ok_or("Node outputs are not available")?;
        let outputs = outputs.lock().map_err(|_| "Node outputs lock failed")?;
        let output = outputs.get(&name).ok_or("Node has not run or is not part of this flow")?;
        serde_json::to_string(output).map_err(|_| "Could not encode node output".into())
    }
}
