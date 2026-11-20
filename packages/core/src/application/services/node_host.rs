use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::ResourceException;
use crate::application::services::flow::FlowService;
use crate::application::services::node::NodeService;
use crate::application::services::secret::SecretService;
use giong_di::Injectable;
use giong_node::host::{Flow, FlowSyncResult, NodeHost, NodeOutput, NodeScope};
use giong_node::storage::Storage;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use tokio::runtime::Handle;
use uuid::Uuid;

#[derive(Injectable)]
pub(crate) struct NodeHostService {
    secret_service: Arc<SecretService>,
    flow_service: Arc<FlowService>,
    node_service: Arc<NodeService>,
}

impl NodeHost for NodeHostService {
    fn storage(&self, scope: &NodeScope) -> Storage {
        self.node_service.storage(scope.node_id)
    }

    fn open_log(&self, scope: &NodeScope) -> anyhow::Result<File> {
        Ok(NodeService::open_log(scope.node_id, scope.execution_id)?)
    }

    fn outputs(&self, scope: &NodeScope) -> anyhow::Result<BTreeMap<String, NodeOutput>> {
        Ok(Handle::try_current()?.block_on(self.node_service.get_flow_outputs(scope.flow_id))?)
    }

    fn get_secret(
        &self,
        scope: &NodeScope,
        vault: Option<&str>,
        key: &str,
    ) -> anyhow::Result<String> {
        let result = Handle::try_current()?.block_on(async {
            let flow = self.flow_service.get_flow(scope.flow_id).await?;
            let vault_ids: Vec<Uuid> = serde_json::from_value(flow.vault_ids)?;
            self.secret_service.get_secret_value(&vault_ids, vault, key).await
        });
        if let Err(
            error
            @ (Exception::BadRequest(_) | Exception::Resource(ResourceException::NotFound(_))),
        ) = &result
            && let Ok(mut logger) = self.open_log(scope)
        {
            let _ = writeln!(logger, "Secret lookup failed: {error}");
        }
        Ok(result?)
    }

    fn emit_changed(&self, scope: &NodeScope) -> anyhow::Result<()> {
        Handle::try_current()?.block_on(async {
            let targets = self.flow_service.get_event_targets(scope.flow_id, scope.node_id).await?;
            for target in targets {
                self.node_service.enqueue_execution(scope.flow_id, target).await?;
            }
            Ok(())
        })
    }

    fn synchronize_flow(
        &self,
        scope: &NodeScope,
        flows: Vec<Flow>,
    ) -> anyhow::Result<FlowSyncResult> {
        Ok(Handle::try_current()?.block_on(self.node_service.synchronize_flow(scope, flows))?)
    }
}
