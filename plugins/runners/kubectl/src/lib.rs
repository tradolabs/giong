wit_bindgen::generate!({ path: "../../../packages/node/wit", world: "node" });

mod configuration;
mod deployment;
mod types;

use giong_node::types::metadata::Metadata;

struct Kubectl;

impl Guest for Kubectl {
    fn metadata() -> Result<String, String> {
        serde_json::to_string(&Metadata {
            kind: "runners/kubectl".into(),
            name: "kubectl".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: "Render a Helm chart from a source node and apply it with kubectl".into(),
        })
        .map_err(|_| "Could not encode metadata".into())
    }

    fn execute(args: String) -> Result<(), String> {
        deployment::execute(&args)
    }
}

export!(Kubectl);
