use crate::application::exceptions::base::Exception;
use giong_node::types::node::{Definition, Flow};
use std::collections::HashSet;

pub(crate) struct FlowNode {
    pub id: String,
    pub definition: Definition,
}

pub(crate) fn flow_nodes(flow: Flow, scope: &str) -> Result<Vec<FlowNode>, Exception> {
    let names: std::collections::HashSet<_> =
        flow.nodes.iter().map(|node| node.name.as_str()).collect();
    for node in &flow.nodes {
        for trigger in &node.triggers {
            match trigger {
                giong_node::types::node::Trigger::Interval { seconds }
                    if *seconds == 0 || *seconds > i64::MAX as u64 =>
                {
                    return Err(Exception::BadRequest(
                        "Trigger interval must be a positive number of seconds".into(),
                    ));
                }
                giong_node::types::node::Trigger::Event { node: source, event }
                    if event.trim().is_empty() || !names.contains(source.as_str()) =>
                {
                    return Err(Exception::BadRequest(
                        "Event triggers must name a node in this flow and a nonempty event".into(),
                    ));
                }
                _ => {}
            }
        }
    }
    // A topological order validates event dependencies and prevents trigger loops.
    let mut remaining = flow.nodes;
    let mut ordered = Vec::new();
    let mut done = HashSet::new();
    while !remaining.is_empty() {
        let Some(index) = remaining.iter().position(|node| {
            node.triggers.iter().all(|trigger| match trigger {
                giong_node::types::node::Trigger::Event { node, .. } => done.contains(node),
                _ => true,
            })
        }) else {
            return Err(Exception::BadRequest("Node triggers contain a cycle".into()));
        };
        let node = remaining.remove(index);
        done.insert(node.name.clone());
        ordered.push(node);
    }
    let mut names = HashSet::new();
    ordered
        .into_iter()
        .map(|definition| {
            let name = &definition.name;
            if name.trim().is_empty()
                || name.chars().any(char::is_control)
                || !names.insert(name.clone())
                || !definition.properties.is_object()
            {
                return Err(Exception::BadRequest(
                    "Nodes need unique names and object properties".into(),
                ));
            }
            giong_node::runtime::path(&definition.kind)
                .map_err(|error| Exception::BadRequest(error.to_string()))?;
            let id =
                definition.id.clone().unwrap_or(serde_json::to_string(&("flow", scope, name))?);
            Ok(FlowNode { id, definition })
        })
        .collect()
}

pub(crate) async fn stored_nodes(
    database: &giong_database::Database,
    flow_id: uuid::Uuid,
) -> Result<Vec<giong_database::entities::node_entity::NodeEntity>, Exception> {
    let mut nodes = database
        .nodes
        .find(
            giong_database::repositories::types::condition::Condition::new().eq("flow_id", flow_id),
        )
        .await?;
    nodes.sort_by_key(|node| node.id);
    Ok(nodes)
}
pub(crate) fn definition(
    node: &giong_database::entities::node_entity::NodeEntity,
) -> Result<Definition, Exception> {
    Ok(Definition {
        id: Some(node.id.to_string()),
        name: node.name.clone(),
        kind: node.kind.clone(),
        properties: node.properties.clone(),
        triggers: serde_json::from_value(node.triggers.clone())?,
    })
}
