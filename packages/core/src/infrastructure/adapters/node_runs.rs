use crate::application::types::node_run::{NodeRun, RunStatus};
use crate::infrastructure::adapters::node_storage::NodeStorage;
use anyhow::Result;
use chrono::Utc;
use giong_di::{Injectable, Resolver};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

static WRITE: Mutex<()> = Mutex::new(());
pub struct NodeRuns {
    storage: Arc<NodeStorage>,
}
impl Injectable for NodeRuns {
    fn inject(resolver: &mut Resolver<'_>) -> Result<Self, giong_di::Error> {
        Ok(Self { storage: resolver.get()? })
    }
}
impl NodeRuns {
    pub fn list(&self) -> Result<Vec<NodeRun>> {
        let mut runs: Vec<NodeRun> = self
            .storage
            .open()?
            .namespace("node-runs")
            .list(b"")?
            .into_iter()
            .map(|(_, bytes)| serde_json::from_slice(&bytes))
            .collect::<Result<_, _>>()?;
        runs.sort_by_key(|run| std::cmp::Reverse(run.id));
        Ok(runs)
    }
    pub fn get(&self, id: Uuid) -> Result<Option<NodeRun>> {
        self.storage
            .open()?
            .namespace("node-runs")
            .get(id.as_bytes())?
            .map(|bytes| serde_json::from_slice(&bytes).map_err(Into::into))
            .transpose()
    }
    pub fn active(&self, flow_id: Uuid) -> Result<bool> {
        Ok(self.list()?.iter().any(|run| run.flow_id == flow_id && run.status.active()))
    }
    pub fn create(&self, flow_id: Uuid, node_id: Uuid) -> Result<NodeRun> {
        let run = NodeRun {
            id: Uuid::now_v7(),
            flow_id,
            node_id,
            status: RunStatus::Pending,
            started_at: None,
            finished_at: None,
            exit_code: None,
            log_timestamp: None,
            created_at: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        self.put(&run)?;
        Ok(run)
    }
    fn put(&self, run: &NodeRun) -> Result<()> {
        Ok(self
            .storage
            .open()?
            .namespace("node-runs")
            .put(run.id.as_bytes(), &serde_json::to_vec(run)?)?)
    }
    pub fn claim(&self, id: Uuid) -> Result<Option<NodeRun>> {
        let _guard = WRITE.lock().map_err(|_| anyhow::anyhow!("Node run lock failed"))?;
        let Some(mut run) = self.get(id)? else {
            return Ok(None);
        };
        if run.status != RunStatus::Pending {
            return Ok(None);
        }
        run.log_timestamp = Some(crate::application::helpers::node_logs::timestamp());
        run.status = RunStatus::Running;
        run.started_at = Some(Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());
        self.put(&run)?;
        let namespace = self.storage.open()?.namespace("node-runs");
        for old in self.list()?.into_iter().filter(|old| old.node_id == run.node_id).skip(10) {
            if !old.status.active() {
                namespace.delete(old.id.as_bytes())?;
            }
        }
        Ok(Some(run))
    }
    pub fn finish(&self, id: Uuid, status: RunStatus, exit_code: Option<i32>) -> Result<()> {
        let _guard = WRITE.lock().map_err(|_| anyhow::anyhow!("Node run lock failed"))?;
        if let Some(mut run) = self.get(id)? {
            run.status = status;
            run.exit_code = exit_code;
            run.finished_at = Some(Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());
            self.put(&run)?;
        }
        Ok(())
    }
    pub fn delete_flow(&self, flow_id: Uuid) -> Result<()> {
        let _guard = WRITE.lock().map_err(|_| anyhow::anyhow!("Node run lock failed"))?;
        let namespace = self.storage.open()?.namespace("node-runs");
        for run in self.list()?.into_iter().filter(|run| run.flow_id == flow_id) {
            namespace.delete(run.id.as_bytes())?;
        }
        Ok(())
    }
}
