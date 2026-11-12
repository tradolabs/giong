use crate::application::exceptions::base::Exception;
use crate::application::helpers::nodes::{flow_nodes, stored_nodes};
use crate::application::helpers::secrets::validate_vaults;
use crate::application::use_cases::use_case::UseCase;
use crate::infrastructure::adapters::node_runs::NodeRuns;
use giong_database::Database;
use giong_database::entities::flow_entity::FlowEntity;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use giong_node::types::node::{Definition, Flow};
use std::{collections::HashSet, sync::Arc};
use uuid::Uuid;

#[derive(Injectable)]
pub struct UpsertFlowUseCase {
    database: Arc<Database>,
    runs: Arc<NodeRuns>,
}
pub struct UpsertFlowInput {
    pub id: Option<Uuid>,
    pub name: String,
    pub nodes: Vec<Definition>,
    pub inherited_vault_ids: Vec<Uuid>,
}
impl UseCase for UpsertFlowUseCase {
    type Input = UpsertFlowInput;
    type Output = FlowEntity;
    async fn execute(&self, input: Self::Input) -> Result<FlowEntity, Exception> {
        let UpsertFlowInput { id, name, nodes, inherited_vault_ids } = input;
        let vault_id = if let Some(id) = id {
            self.database
                .flows
                .find_one(Condition::new().eq("id", id))
                .await?
                .ok_or(Exception::NotFound("modules.flow.not_found"))?
                .vault_id
        } else {
            None
        };
        validate_vaults(&self.database, vault_id, &inherited_vault_ids).await?;
        let name = name.trim().to_owned();
        if name.is_empty() || name.chars().any(char::is_control) {
            return Err(Exception::BadRequest(
                "Flow name is required and cannot contain control characters".into(),
            ));
        }
        if let Some(existing) =
            self.database.flows.find_one(Condition::new().eq("name", name.as_str())).await?
        {
            if Some(existing.id) != id {
                return Err(Exception::Conflict("modules.flow.already_exists"));
            }
        }
        if let Some(id) = id {
            if !self.database.flows.exists(Condition::new().eq("id", id)).await? {
                return Err(Exception::NotFound("modules.flow.not_found"));
            }
            if self.runs.active(id)? {
                return Err(Exception::Conflict("modules.flow.active_run_exists"));
            }
        }
        flow_nodes(
            Flow {
                name: Some(name.clone()),
                nodes: nodes.clone(),
                vault_id: None,
                inherited_vault_ids: vec![],
            },
            &name,
        )?;
        let previous =
            if let Some(id) = id { stored_nodes(&self.database, id).await? } else { vec![] };
        let mut seen = HashSet::new();
        let mut planned = Vec::new();
        for node in nodes {
            let existing = if let Some(raw) = &node.id {
                let node_id = Uuid::parse_str(raw)
                    .map_err(|_| Exception::BadRequest("Invalid node ID".into()))?;
                Some(
                    previous
                        .iter()
                        .find(|item| item.id == node_id)
                        .ok_or(Exception::NotFound("Node does not belong to this flow"))?,
                )
            } else {
                previous.iter().find(|item| item.name == node.name)
            };
            let node_id = existing.map(|node| node.id).unwrap_or_else(Uuid::now_v7);
            if !seen.insert(node_id) {
                return Err(Exception::BadRequest("Duplicate node ID".into()));
            }
            planned.push((node_id, existing.is_some(), node));
        }
        let inherited = serde_json::to_value(inherited_vault_ids)?;
        let mut flow = self
            .database
            .transaction(async || {
                let vault_id = if let Some(vault_id) = vault_id {
                    vault_id
                } else {
                    let base = format!("{name} secrets");
                    let mut vault_name = base.clone();
                    let mut suffix = 2;
                    while self
                        .database
                        .vaults
                        .exists(Condition::new().eq("name", vault_name.as_str()))
                        .await?
                    {
                        vault_name = format!("{base} ({suffix})");
                        suffix += 1;
                    }
                    self.database
                        .vaults
                        .create_one(Document::new().set("name", vault_name))
                        .await?
                        .id
                };
                let changes = Document::new()
                    .set("name", name)
                    .set("vault_id", vault_id)
                    .set("inherited_vault_ids", inherited);
                let flow = if let Some(id) = id {
                    self.database.flows.update_one(Condition::new().eq("id", id), changes).await?;
                    self.database.flows.find_one(Condition::new().eq("id", id)).await?.ok_or_else(
                        || giong_database::error::Error::Unexpected("Flow no longer exists".into()),
                    )?
                } else {
                    self.database.flows.create_one(changes).await?
                };
                // Remove omitted nodes and temporarily free names so existing nodes can be renamed/swapped.
                for old in &previous {
                    if !seen.contains(&old.id) {
                        self.database.nodes.delete_one(Condition::new().eq("id", old.id)).await?;
                    } else {
                        self.database
                            .nodes
                            .update_one(
                                Condition::new().eq("id", old.id),
                                Document::new().set("name", format!("__upsert:{}", old.id)),
                            )
                            .await?;
                    }
                }
                for (node_id, exists, node) in planned {
                    let triggers = serde_json::to_value(node.triggers).map_err(|error| {
                        giong_database::error::Error::Unexpected(error.to_string())
                    })?;
                    let changes = Document::new()
                        .set("flow_id", flow.id)
                        .set("name", node.name)
                        .set("kind", node.kind)
                        .set("properties", node.properties)
                        .set("triggers", triggers);
                    if exists {
                        self.database
                            .nodes
                            .update_one(Condition::new().eq("id", node_id), changes)
                            .await?;
                    } else {
                        self.database.nodes.create_one(changes.set("id", node_id)).await?;
                    }
                }
                Ok(flow)
            })
            .await?;
        flow.nodes = stored_nodes(&self.database, flow.id).await?;
        Ok(flow)
    }
}
