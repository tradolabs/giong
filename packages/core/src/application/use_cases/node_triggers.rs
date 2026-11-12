use crate::application::exceptions::base::Exception;
use crate::application::helpers::nodes::definition;
use crate::application::ports::queue::{events::RunNodeEvent, queue_manager::QueueManager};
use crate::application::types::node_run::RunStatus;
use crate::application::use_cases::create_node_run::{CreateNodeRunInput, CreateNodeRunUseCase};
use crate::application::use_cases::use_case::UseCase;
use crate::infrastructure::adapters::node_runs::NodeRuns;
use crate::infrastructure::adapters::node_storage::NodeStorage;
use chrono::{NaiveDateTime, Utc};
use giong_database::Database;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use giong_node::types::node::Trigger;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
struct NodeEvent {
    source: String,
    scope: String,
    event: String,
}

#[derive(Injectable)]
pub struct DispatchNodeTriggersUseCase {
    database: Arc<Database>,
    create_run: Arc<CreateNodeRunUseCase>,
    runs: Arc<NodeRuns>,
    storage: Arc<NodeStorage>,
}
impl UseCase for DispatchNodeTriggersUseCase {
    type Input = ();
    type Output = ();
    async fn execute(&self, _: ()) -> Result<(), Exception> {
        let nodes = self.database.nodes.find(Condition::new()).await?;
        if nodes.is_empty() {
            return Ok(());
        }
        let storage = self.storage.clone();
        let events = tokio::task::spawn_blocking(move || {
            storage.open().map(|kv| kv.namespace("node-events"))
        })
        .await
        .map_err(Exception::unexpected)??;
        for (key, bytes) in events.list(b"").map_err(Exception::unexpected)? {
            let message: NodeEvent = serde_json::from_slice(&bytes)?;
            if let (Ok(source_id), Ok(flow_id)) =
                (Uuid::parse_str(&message.source), Uuid::parse_str(&message.scope))
            {
                if let Some(source) =
                    nodes.iter().find(|node| node.id == source_id && node.flow_id == flow_id)
                {
                    for node in nodes.iter().filter(|node| node.flow_id == flow_id) {
                        let triggers = definition(node)?.triggers;
                        if triggers.iter().any(|trigger| matches!(trigger, Trigger::Event { node, event } if node == &source.name && event == &message.event)) {
                            self.create_run.submit(CreateNodeRunInput { flow_id, node_id: node.id }).await?;
                        }
                    }
                }
            }
            events.delete(&key).map_err(Exception::unexpected)?;
        }
        let now = Utc::now().timestamp();
        for node in nodes {
            let triggers = definition(&node)?.triggers;
            let interval = triggers
                .iter()
                .filter_map(|trigger| match trigger {
                    Trigger::Interval { seconds } => Some(*seconds as i64),
                    _ => None,
                })
                .min();
            let Some(interval) = interval else {
                continue;
            };
            let runs: Vec<_> =
                self.runs.list()?.into_iter().filter(|run| run.node_id == node.id).collect();
            let latest = runs.iter().max_by_key(|run| run.id);
            let due = latest
                .map(|run| {
                    NaiveDateTime::parse_from_str(&run.created_at, "%Y-%m-%d %H:%M:%S")
                        .map(|time| time.and_utc().timestamp().saturating_add(interval) <= now)
                        .map_err(Exception::unexpected)
                })
                .transpose()?
                .unwrap_or(true);
            if due {
                self.create_run
                    .submit(CreateNodeRunInput { flow_id: node.flow_id, node_id: node.id })
                    .await?;
            }
        }
        Ok(())
    }
}

#[derive(Injectable)]
pub struct ResumePendingRunsUseCase {
    runs: Arc<NodeRuns>,
    queue: Arc<dyn QueueManager>,
}
impl UseCase for ResumePendingRunsUseCase {
    type Input = ();
    type Output = ();
    async fn execute(&self, _: ()) -> Result<(), Exception> {
        for run in self.runs.list()? {
            if run.status == RunStatus::Running {
                self.runs.finish(run.id, RunStatus::Interrupted, None)?;
            } else if run.status == RunStatus::Pending {
                self.queue.enqueue(RunNodeEvent { run_id: run.id }.into()).await?;
            }
        }
        Ok(())
    }
}

#[derive(Injectable)]
pub struct TriggerNodeUseCase {
    database: Arc<Database>,
    create_run: Arc<CreateNodeRunUseCase>,
}
impl UseCase for TriggerNodeUseCase {
    type Input = (Uuid, Uuid);
    type Output = crate::application::types::node_run::NodeRun;
    async fn execute(&self, (flow_id, node_id): Self::Input) -> Result<Self::Output, Exception> {
        if !self
            .database
            .nodes
            .exists(Condition::new().eq("id", node_id).eq("flow_id", flow_id))
            .await?
        {
            return Err(Exception::NotFound("Node not found in flow"));
        }
        self.create_run.execute(CreateNodeRunInput { flow_id, node_id }).await
    }
}
