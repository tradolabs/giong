use crate::helpers::parse_flow;
use crate::types::FlowProperties;
use anyhow::Result;
use giong_node::sdk::{Context, Plugin};
use giong_node::types::metadata::Metadata;
use giong_plugins_repository::RepositoryOutput;
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

    fn execute(ctx: &mut Context, properties: FlowProperties) -> Result<()> {
        let source: RepositoryOutput = ctx.get_output(&properties.source)?;
        let workspace = ctx.get_source_workspace(&properties.source)?;
        let repository = giong_git::workspace_checkout(
            Path::new(&workspace),
            Path::new(&source.repository_path),
        )?;
        let manifests = giong_git::read_manifests(&repository, Path::new(&properties.path))?;
        let mut definitions = Vec::new();
        for (path, content) in manifests {
            if let Some(flow) = parse_flow(&path, &content)? {
                definitions.push(flow);
            }
        }

        ctx.log.info(format!("Synchronizing {} flow definitions.", definitions.len()))?;
        let report = ctx.synchronize_flow(definitions)?;
        ctx.save_output(&report)?;
        ctx.emit_changed()
    }
}
