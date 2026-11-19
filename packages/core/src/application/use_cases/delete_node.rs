use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::application::services::flow::FlowService;
use crate::application::services::node::NodeService;
use crate::application::use_cases::use_case::UseCase;
use crate::config::Config;
use giong_database::Database;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use giong_kv::kv::Kv;
use giong_node::host::Trigger;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Injectable)]
pub struct DeleteNodeUseCase {
    database: Arc<Database>,
    kv: Arc<Kv>,
    node_service: Arc<NodeService>,
    flow_service: Arc<FlowService>,
}

impl UseCase for DeleteNodeUseCase {
    type Input = (Uuid, Uuid);
    type Output = ();

    async fn execute(&self, (flow_id, node_id): Self::Input) -> Result<(), Exception> {
        let node = self
            .database
            .nodes
            .find_one(Condition::new().eq("flow_id", flow_id).eq("id", node_id))
            .await?
            .ok_or(ResourceException::NotFound(Resource::Node))?;
        let mut updates = Vec::new();
        for other in self
            .database
            .nodes
            .find(Condition::new().eq("flow_id", flow_id).ne("id", node_id))
            .await?
        {
            let mut triggers: Vec<Trigger> = serde_json::from_value(other.triggers)?;
            let count = triggers.len();
            triggers.retain(|trigger| !matches!(trigger, Trigger::Event { node: source, .. } if source == &node.name));
            if triggers.len() != count {
                updates.push((other.id, serde_json::to_value(triggers)?));
            }
        }
        self.database
            .transaction(async || {
                for (id, triggers) in updates {
                    self.database
                        .nodes
                        .update_one(
                            Condition::new().eq("flow_id", flow_id).eq("id", id),
                            Document::new().set("triggers", triggers),
                        )
                        .await?;
                }
                self.database
                    .nodes
                    .delete_one(Condition::new().eq("flow_id", flow_id).eq("id", node_id))
                    .await?;
                Ok(())
            })
            .await?;
        self.node_service.unschedule_node(node_id).await?;
        self.flow_service.invalidate_graph(flow_id).await;
        if let Err(error) = giong_node::runtime::remove_workspace(flow_id, node_id).await {
            eprintln!("Could not remove node workspace {node_id}: {error}");
        }
        let kv = self.kv.clone();
        let cleanup = tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            kv.namespace(&format!("nodes/{node_id}")).truncate()?;
            let data_dir = Config::data_dir();
            for directory in [
                NodeService::node_logs_dir(node_id),
                data_dir
                    .join("workspaces")
                    .join(flow_id.to_string())
                    .join(format!("node-{node_id}")),
            ] {
                match std::fs::remove_dir_all(&directory) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => eprintln!("Could not remove {}: {error}", directory.display()),
                }
            }
            Ok(())
        })
        .await;
        match cleanup {
            Ok(Ok(())) => {}
            Ok(Err(error)) => eprintln!("Could not clean up deleted node: {error}"),
            Err(error) => eprintln!("Could not clean up deleted node: {error}"),
        }
        Ok(())
    }
}
