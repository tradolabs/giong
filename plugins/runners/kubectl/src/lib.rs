wit_bindgen::generate!({
    path: "../../../packages/runner/wit",
    world: "runner",
});

use giong::runner::types::{DockerPlan, Metadata, Plan};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Configurations {
    namespace: Option<String>,
    release: Option<String>,
    values_file: Option<String>,
    source: Sources,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    repo_url: String,
    #[serde(rename = "revision")]
    _revision: Option<String>,
    path: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Sources {
    One(Source),
    Many(Vec<Source>),
}

struct KubectlRunner;

impl Guest for KubectlRunner {
    fn metadata() -> Result<Metadata, String> {
        let source_schema = json!({
            "type": "object", "additionalProperties": false,
            "required": ["repo_url", "path"],
            "description": "Chart source with repo_url, revision and path; credentials come from the host. May also be a list of sources.",
            "properties": {
                "repo_url": { "type": "string", "title": "Repository URL" },
                "revision": { "type": "string", "default": "HEAD" },
                "path": { "type": "string", "title": "Chart path", "description": "Chart folder relative to the source repository root" }
            }
        });
        Ok(Metadata {
            id: "kubectl".into(),
            name: "kubectl".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description:
                "Renders a Helm chart and applies it with kubectl. Set up Kubernetes access in Secrets.".into(),
            defaults: json!({}).to_string(),
            configuration_schema: json!({
                "type": "object", "additionalProperties": false, "required": ["source"],
                "properties": {
                    "source": { "oneOf": [source_schema.clone(), {"type": "array", "minItems": 1, "items": source_schema}] },
                    "namespace": { "type": "string", "title": "Namespace" },
                    "release": { "type": "string", "title": "Release name (Defaults to the stack name)" },
                    "values_file": { "type": "string", "minLength": 1, "title": "Values file", "description": "Optional values file relative to each chart folder" }
                }
            }).to_string(),
        })
    }

    fn plan(stack_name: String, configurations: String) -> Result<Plan, String> {
        let config: Configurations =
            serde_json::from_str(&configurations).map_err(|error| error.to_string())?;
        let release = config.release.as_deref().unwrap_or(&stack_name);
        let namespace = config.namespace.as_deref().unwrap_or("default");
        let sources = match config.source {
            Sources::One(source) => vec![source],
            Sources::Many(sources) => sources,
        };
        if sources.is_empty() {
            return Err("At least one source is required".into());
        }
        let mut args =
            vec![release.into(), namespace.into(), config.values_file.unwrap_or_default()];
        for (index, source) in sources.iter().enumerate() {
            if source.repo_url.trim().is_empty() || source.path.trim().is_empty() {
                return Err("source.repo_url and source.path are required".into());
            }
            args.push(format!("/workspace/source-{index}/{}", source.path));
        }
        Ok(Plan::Docker(DockerPlan {
            dockerfile: include_str!("../Dockerfile").into(),
            dockerignore: None,
            args,
        }))
    }
}

export!(KubectlRunner);
