use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::application::services::flow::FlowService;
use crate::application::services::node::NodeService;
use crate::application::use_cases::use_case::UseCase;
use crate::config::Config;
use giong_database::Database;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use giong_kv::kv::Kv;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Injectable)]
pub struct DeleteFlowUseCase {
    kv: Arc<Kv>,
    database: Arc<Database>,
    flow_service: Arc<FlowService>,
    node_service: Arc<NodeService>,
}

impl UseCase for DeleteFlowUseCase {
    type Input = Uuid;
    type Output = ();
    async fn execute(&self, id: Uuid) -> Result<(), Exception> {
        let mut flow = self
            .database
            .flows
            .find_one(Condition::new().eq("id", id))
            .await?
            .ok_or(ResourceException::NotFound(Resource::Flow))?;
        self.flow_service.populate_nodes(std::slice::from_mut(&mut flow)).await?;
        let nodes = flow.nodes;
        self.database.flows.delete_one(Condition::new().eq("id", id)).await?;
        for node in &nodes {
            self.node_service.unschedule_node(node.id).await?;
        }
        self.flow_service.invalidate_graph(id).await;
        let cleanup_workspace = async {
            let images: Vec<_> = self
                .database
                .plugins
                .find(Condition::new())
                .await?
                .into_iter()
                .map(|plugin| plugin.image_id)
                .collect();
            giong_node::runtime::remove_flow_workspace(id, &images).await?;
            Ok::<(), Exception>(())
        };
        if let Err(error) = cleanup_workspace.await {
            eprintln!("Could not remove flow workspace {id}: {error}");
        }
        let kv = self.kv.clone();
        let cleanup = tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            let scope = id.to_string();
            let workspace = Config::data_dir().join("workspaces").join(&scope);
            if workspace.exists() {
                if let Err(error) = std::fs::remove_dir_all(&workspace) {
                    eprintln!("Could not remove {}: {error}", workspace.display());
                }
            }
            for node in nodes {
                kv.namespace(&format!("nodes/{}", node.id)).truncate()?;
            }
            Ok(())
        })
        .await;
        match cleanup {
            Ok(Ok(())) => {}
            Ok(Err(error)) => eprintln!("Could not clean up deleted flow: {error}"),
            Err(error) => eprintln!("Could not clean up deleted flow: {error}"),
        }
        Ok(())
    }
}
