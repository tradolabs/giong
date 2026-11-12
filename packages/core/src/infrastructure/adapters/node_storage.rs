use anyhow::{Result, anyhow};
use giong_kv::kv::Kv;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub struct NodeStorage {
    path: PathBuf,
    flows: Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>,
    database: Mutex<Option<Kv>>,
}

impl NodeStorage {
    pub fn new(path: PathBuf) -> Self {
        Self { path, database: Mutex::new(None), flows: Mutex::new(HashMap::new()) }
    }

    pub fn flow_lock(&self, id: &str) -> Result<Arc<tokio::sync::Mutex<()>>> {
        let mut flows = self.flows.lock().map_err(|_| anyhow!("Flow lock failed"))?;
        Ok(flows.entry(id.to_owned()).or_default().clone())
    }
    pub fn open(&self) -> Result<Kv> {
        let mut database = self.database.lock().map_err(|_| anyhow!("Node storage lock failed"))?;
        if let Some(database) = database.as_ref() {
            return Ok(database.clone());
        }
        let opened = Kv::new(&self.path)?;
        *database = Some(opened.clone());
        Ok(opened)
    }
}
