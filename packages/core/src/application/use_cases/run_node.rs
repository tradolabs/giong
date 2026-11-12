use crate::application::exceptions::base::Exception;
use crate::application::helpers::{
    node_logs,
    nodes::{definition, stored_nodes},
};
use crate::application::types::node_run::RunStatus;
use crate::application::use_cases::{
    run_flow::{RunFlowInput, RunFlowUseCase},
    use_case::UseCase,
};
use crate::infrastructure::adapters::node_runs::NodeRuns;
use giong_database::Database;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use giong_node::types::node::Flow;
use std::{io::Write, sync::Arc};
use uuid::Uuid;
pub struct RunNodeInput {
    pub run_id: Uuid,
}
#[derive(Injectable)]
pub struct RunNodeUseCase {
    database: Arc<Database>,
    runs: Arc<NodeRuns>,
    flow: Arc<RunFlowUseCase>,
}
impl UseCase for RunNodeUseCase {
    type Input = RunNodeInput;
    type Output = ();
    async fn execute(&self, input: Self::Input) -> Result<(), Exception> {
        let Some(run) = self.runs.claim(input.run_id)? else {
            return Ok(());
        };
        let timestamp = run
            .log_timestamp
            .as_deref()
            .ok_or_else(|| Exception::unexpected("Node log timestamp is missing"))?;
        let log_path = node_logs::path(run.node_id, timestamp);
        let result = async {
            drop(node_logs::open(run.node_id, timestamp)?);
            let flow = self
                .database
                .flows
                .find_one(Condition::new().eq("id", run.flow_id))
                .await?
                .ok_or(Exception::NotFound("modules.flow.not_found"))?;
            let nodes = stored_nodes(&self.database, flow.id)
                .await?
                .iter()
                .map(definition)
                .collect::<Result<Vec<_>, _>>()?;
            self.flow
                .execute(RunFlowInput {
                    flow: Flow {
                        name: Some(flow.name),
                        nodes,
                        vault_id: flow.vault_id.map(|id| id.to_string()),
                        inherited_vault_ids: serde_json::from_value::<Vec<Uuid>>(
                            flow.inherited_vault_ids,
                        )?
                        .into_iter()
                        .map(|id| id.to_string())
                        .collect(),
                    },
                    scope: Some(flow.id.to_string()),
                    run_id: Some(run.id),
                    log_timestamp: run.log_timestamp.clone(),
                    node_id: Some(run.node_id),
                })
                .await
        }
        .await;
        if let Err(error) = &result {
            if let Ok(mut file) = std::fs::File::options().create(true).append(true).open(&log_path)
            {
                let _ = writeln!(file, "{error}");
            }
        }
        self.runs.finish(
            run.id,
            if result.is_ok() { RunStatus::Succeeded } else { RunStatus::Failed },
            Some(if result.is_ok() { 0 } else { 1 }),
        )?;
        Ok(())
    }
}
