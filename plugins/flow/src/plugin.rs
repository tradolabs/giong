use crate::types::FlowProperties;
use anyhow::{Result, anyhow};
use giong_node::sdk::{Client, Plugin};
use giong_node::types::metadata::Metadata;
use giong_plugins_repository::RepositoryOutput;
use serde_yaml_ng::Value;
use std::path::Path;

const FLOW_RUNNER_KIND: &str = "flow";

pub(crate) struct Flow;

impl Plugin for Flow {
    type Input = FlowProperties;

    fn metadata() -> Metadata {
        Metadata {
            kind: FLOW_RUNNER_KIND.into(),
            name: "Flow".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: "Synchronize flow definitions from a repository node".into(),
        }
    }

    fn execute(client: &mut Client<'_>, properties: FlowProperties) -> Result<()> {
        let source: RepositoryOutput = client.get_output(&properties.source)?;
        let workspace = client.get_source_workspace(&properties.source)?;
        let repository = giong_git::workspace_checkout(
            Path::new(&workspace),
            Path::new(&source.repository_path),
        )?;
        let manifests = giong_git::read_manifests(&repository, Path::new(&properties.path))?;
        let mut definitions = Vec::new();
        for (path, content) in manifests {
            if let Some(manifest) = parse_flow(&path, &content)? {
                definitions.push(
                    serde_yaml_ng::from_value(manifest)
                        .map_err(|error| anyhow!("{path}: {error}"))?,
                );
            }
        }

        eprintln!("Synchronizing {} flow definitions.", definitions.len());
        let report = client.synchronize_flow(definitions)?;
        client.save_output(&report)?;
        client.emit_changed()
    }
}

fn parse_flow(path: &str, content: &str) -> Result<Option<Value>> {
    let value: Value =
        serde_yaml_ng::from_str(content).map_err(|error| anyhow!("{path}: {error}"))?;
    let mut manifest = if let Some(flow) = value.get("flow") {
        flow.clone()
    } else if value.get("nodes").is_some() {
        value
    } else {
        return Ok(None);
    };
    if let Some(fields) = manifest.as_mapping_mut()
        && fields.get("name").is_none_or(Value::is_null)
        && let Some(name) = Path::new(path).file_stem().and_then(|name| name.to_str())
    {
        fields.insert(Value::String("name".into()), Value::String(name.into()));
    }
    Ok(Some(manifest))
}
