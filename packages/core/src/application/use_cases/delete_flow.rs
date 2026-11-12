use crate::application::exceptions::base::Exception;
use crate::application::helpers::filesystem::{remove_dir, root_dir};
use crate::application::helpers::nodes::stored_nodes;
use crate::application::use_cases::use_case::UseCase;
use crate::infrastructure::adapters::node_runs::NodeRuns;
use crate::infrastructure::adapters::node_storage::NodeStorage;
use giong_database::Database;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Injectable)]
pub struct DeleteFlowUseCase {
    database: Arc<Database>,
    runs: Arc<NodeRuns>,
    storage: Arc<NodeStorage>,
}
impl UseCase for DeleteFlowUseCase {
    type Input = Uuid;
    type Output = ();
    async fn execute(&self, id: Uuid) -> Result<(), Exception> {
        let lock = self.storage.flow_lock(&id.to_string())?;
        let _guard = lock.lock().await;
        let flow = self
            .database
            .flows
            .find_one(Condition::new().eq("id", id))
            .await?
            .ok_or(Exception::NotFound("modules.flow.not_found"))?;
        if self.runs.active(id)? {
            return Err(Exception::Conflict("modules.flow.active_run_exists"));
        }
        if let Some(vault_id) = flow.vault_id {
            for other in self.database.flows.find(Condition::new()).await? {
                if other.id == id {
                    continue;
                }
                let inherited: Vec<Uuid> = serde_json::from_value(other.inherited_vault_ids)?;
                if other.vault_id == Some(vault_id) || inherited.contains(&vault_id) {
                    return Err(Exception::Conflict("modules.flow.vault_in_use"));
                }
            }
        }
        let nodes = stored_nodes(&self.database, id).await?;
        self.database
            .transaction(async || {
                for node in &nodes {
                    self.database.nodes.delete_one(Condition::new().eq("id", node.id)).await?;
                }
                self.database.flows.delete_one(Condition::new().eq("id", id)).await?;
                if let Some(vault_id) = flow.vault_id {
                    for secret in self
                        .database
                        .secrets
                        .find(Condition::new().eq("vault_id", vault_id))
                        .await?
                    {
                        self.database
                            .secrets
                            .delete_one(Condition::new().eq("id", secret.id))
                            .await?;
                    }
                    self.database.vaults.delete_one(Condition::new().eq("id", vault_id)).await?;
                }
                Ok(())
            })
            .await?;
        self.runs.delete_flow(id)?;
        let storage = self.storage.clone();
        let cleanup = tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            remove_dir(&root_dir().join("workspaces").join(id.to_string()));
            if nodes.is_empty() {
                return Ok(());
            }
            let kv = storage.open()?;
            for node in nodes {
                let namespace = kv.namespace(&node.id.to_string());
                for (key, _) in namespace.list(b"")? {
                    namespace.delete(&key)?;
                }
            }
            let events = kv.namespace("node-events");
            for (key, bytes) in events.list(b"")? {
                let event: serde_json::Value = serde_json::from_slice(&bytes)?;
                if event.get("scope").and_then(serde_json::Value::as_str)
                    == Some(id.to_string().as_str())
                {
                    events.delete(&key)?;
                }
            }
            Ok(())
        })
        .await;
        match cleanup {
            Ok(Ok(())) => {}
            Ok(Err(error)) => eprintln!("Could not clean up deleted flow runtime: {error}"),
            Err(error) => eprintln!("Could not clean up deleted flow runtime: {error}"),
        }
        Ok(())
    }
}
