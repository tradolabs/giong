use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::application::ports::queue::{events::ExecuteNodeEvent, queue_manager::QueueManager};
use crate::application::types::node_execution::{ExecutionStatus, NodeExecution};
use giong_database::Database;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use giong_kv::kv::Kv;
use giong_node::types::node::Trigger;
use giong_scheduler::IntervalScheduler;
use std::borrow::Cow;
use std::cmp::Reverse;
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Injectable)]
pub struct NodeService {
    database: Arc<Database>,
    kv: Arc<Kv>,
    queue: Arc<dyn QueueManager>,
    scheduler: Arc<IntervalScheduler>,
}

impl NodeService {
    pub(crate) async fn enqueue_execution(
        &self,
        flow_id: Uuid,
        node_id: Uuid,
    ) -> Result<NodeExecution, Exception> {
        if !self
            .database
            .nodes
            .exists(Condition::new().eq("id", node_id).eq("flow_id", flow_id))
            .await?
        {
            return Err(ResourceException::NotFound(Resource::Node).into());
        }
        let execution = NodeExecution {
            id: Uuid::now_v7(),
            flow_id,
            node_id,
            status: ExecutionStatus::Pending,
            started_at: None,
            finished_at: None,
            exit_code: None,
            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        self.kv
            .namespace(&format!("nodes/{node_id}"))
            .put(
                format!("executions/{}", execution.id).as_bytes(),
                &serde_json::to_vec(&execution)?,
            )?;
        self.queue.enqueue(ExecuteNodeEvent { node_id }.into()).await?;
        Ok(execution)
    }

    pub(crate) fn get_executions(&self, node_id: Uuid) -> Result<Vec<NodeExecution>, Exception> {
        let mut executions = self
            .kv
            .namespace(&format!("nodes/{node_id}"))
            .search(b"executions/")?
            .into_iter()
            .map(|(_, bytes)| serde_json::from_slice::<NodeExecution>(&bytes))
            .collect::<Result<Vec<_>, _>>()?;
        executions.sort_by_key(|execution| Reverse(execution.id));
        Ok(executions)
    }

    pub(crate) async fn find_execution(
        &self,
        execution_id: Uuid,
    ) -> Result<Option<NodeExecution>, Exception> {
        let key = format!("executions/{execution_id}");
        for node in self.database.nodes.find(Condition::new()).await? {
            match self.kv.namespace(&format!("nodes/{}", node.id)).get(key.as_bytes()) {
                Ok(bytes) => return Ok(Some(serde_json::from_slice(&bytes)?)),
                Err(giong_kv::error::Error::NotFound) => continue,
                Err(error) => return Err(error.into()),
            }
        }
        Ok(None)
    }

    pub(crate) async fn sync_interval_trigger(
        &self,
        flow_id: Uuid,
        node_id: Uuid,
        triggers: &[Trigger],
    ) -> Result<(), Exception> {
        let interval = triggers
            .iter()
            .filter_map(|trigger| match trigger {
                Trigger::Interval { seconds } => Some(*seconds),
                _ => None,
            })
            .min();
        let Some(seconds) = interval else {
            self.scheduler.unschedule(node_id).await?;
            return Ok(());
        };

        let service = self.clone();
        self.scheduler
            .schedule(node_id, seconds, move || {
                let service = service.clone();
                async move {
                    if let Err(error) = service.enqueue_execution(flow_id, node_id).await {
                        eprintln!("Node interval trigger ({node_id}): {error}");
                    }
                }
            })
            .await?;
        Ok(())
    }

    pub(crate) async fn unschedule_node(&self, node_id: Uuid) -> Result<(), Exception> {
        self.scheduler.unschedule(node_id).await?;
        Ok(())
    }

    pub(crate) async fn sync_all_interval_triggers(&self) -> Result<(), Exception> {
        for node in self.database.nodes.find(Condition::new()).await? {
            let triggers: Vec<Trigger> = serde_json::from_value(node.triggers)?;
            self.sync_interval_trigger(node.flow_id, node.id, &triggers).await?;
        }
        Ok(())
    }

    pub(crate) async fn validate_flow_nodes(
        &self,
        flow_id: Uuid,
        exclude_id: Option<Uuid>,
        name: &str,
        kind: &str,
        properties: &serde_json::Value,
        triggers: &[Trigger],
    ) -> Result<(), Exception> {
        let existing = self.database.nodes.find(Condition::new().eq("flow_id", flow_id)).await?;
        let mut nodes = Vec::with_capacity(existing.len() + 1);
        for node in existing.iter().filter(|node| Some(node.id) != exclude_id) {
            let node_triggers: Vec<Trigger> = serde_json::from_value(node.triggers.clone())?;
            nodes.push((
                node.name.as_str(),
                node.kind.as_str(),
                &node.properties,
                Cow::Owned(node_triggers),
            ));
        }
        nodes.push((name, kind, properties, Cow::Borrowed(triggers)));

        let mut names = HashSet::with_capacity(nodes.len());
        for (node_name, node_kind, node_properties, _) in &nodes {
            if node_name.trim().is_empty()
                || node_name.chars().any(char::is_control)
                || !names.insert(*node_name)
                || !node_properties.is_object()
            {
                return Err(Exception::BadRequest(
                    "Nodes need unique names and object properties".into(),
                ));
            }
            giong_node::runtime::path(node_kind)
                .map_err(|error| Exception::BadRequest(error.to_string()))?;
        }

        for (_, _, _, node_triggers) in &nodes {
            for trigger in node_triggers.iter() {
                match trigger {
                    Trigger::Interval { seconds }
                        if *seconds == 0 || *seconds > i64::MAX as u64 =>
                    {
                        return Err(Exception::BadRequest(
                            "Trigger interval must be a positive number of seconds".into(),
                        ));
                    }
                    Trigger::Event { node: source, event }
                        if event.trim().is_empty() || !names.contains(source.as_str()) =>
                    {
                        return Err(Exception::BadRequest(
                            "Event triggers must name a node in this flow and a nonempty event"
                                .into(),
                        ));
                    }
                    _ => {}
                }
            }
        }

        // A topological order validates event dependencies and prevents trigger loops.
        let mut remaining: Vec<_> =
            nodes.iter().map(|(name, _, _, triggers)| (*name, triggers.as_ref())).collect();
        let mut done = HashSet::with_capacity(remaining.len());
        while !remaining.is_empty() {
            let Some(index) = remaining.iter().position(|(_, node_triggers)| {
                node_triggers.iter().all(|trigger| match trigger {
                    Trigger::Event { node: source, .. } => done.contains(source.as_str()),
                    _ => true,
                })
            }) else {
                return Err(Exception::BadRequest("Node triggers contain a cycle".into()));
            };
            let (node_name, _) = remaining.swap_remove(index);
            done.insert(node_name);
        }

        Ok(())
    }
}
