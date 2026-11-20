use anyhow::{Result, anyhow};
use giong_node::host::Flow;
use serde_yaml_ng::Value;
use std::path::Path;

pub(super) fn parse_flow(path: &str, content: &str) -> Result<Option<Flow>> {
    let mut value: Value =
        serde_yaml_ng::from_str(content).map_err(|error| anyhow!("{path}: {error}"))?;
    let manifest = match value.as_mapping_mut().and_then(|fields| fields.remove("flow")) {
        Some(flow) => flow,
        None if value.get("nodes").is_some() => value,
        None => return Ok(None),
    };
    let is_mapping = manifest.is_mapping();
    let mut flow: Flow =
        serde_yaml_ng::from_value(manifest).map_err(|error| anyhow!("{path}: {error}"))?;
    if is_mapping && flow.name.is_none() {
        flow.name = Path::new(path).file_stem().and_then(|name| name.to_str()).map(str::to_owned);
    }
    Ok(Some(flow))
}
