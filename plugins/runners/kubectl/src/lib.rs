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
    helm_folder: Option<String>,
    values_file: Option<String>,
}

struct KubectlRunner;

impl Guest for KubectlRunner {
    fn metadata() -> Result<Metadata, String> {
        Ok(Metadata {
            id: "kubectl".into(),
            name: "kubectl".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description:
                "Renders a Helm chart and applies it with kubectl. Set up Kubernetes access in Secrets.".into(),
            defaults: json!({ "helm_folder": "helm" }).to_string(),
            configuration_schema: json!({
                "type": "object", "additionalProperties": false,
                "properties": {
                    "helm_folder": { "type": "string", "title": "Helm folder", "default": "helm", "description": "Chart folder relative to the stack folder; use . for the stack folder itself" },
                    "namespace": { "type": "string", "title": "Namespace" },
                    "release": { "type": "string", "title": "Release name (Defaults to the stack name)" },
                    "values_file": { "type": "string", "minLength": 1, "title": "Values file", "description": "Optional path relative to the stack folder, for example values.production.yaml" }
                }
            }).to_string(),
        })
    }

    fn plan(stack_name: String, configurations: String) -> Result<Plan, String> {
        let config: Configurations =
            serde_json::from_str(&configurations).map_err(|error| error.to_string())?;
        let release = config.release.as_deref().unwrap_or(&stack_name);
        let namespace = config.namespace.as_deref().unwrap_or("default");
        let helm_folder = config.helm_folder.as_deref().unwrap_or("helm");
        let mut args = vec![
            release.into(),
            format!("/workspace/{helm_folder}"),
            "--namespace".into(),
            namespace.into(),
            "--no-hooks".into(),
        ];
        if let Some(values_file) = config.values_file {
            args.extend(["--values".into(), format!("/workspace/{values_file}")]);
        }
        Ok(Plan::Docker(DockerPlan {
            dockerfile: include_str!("../Dockerfile").into(),
            dockerignore: None,
            args,
        }))
    }
}

export!(KubectlRunner);
