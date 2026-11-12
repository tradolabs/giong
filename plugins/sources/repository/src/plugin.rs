wit_bindgen::generate!({ path: "../../../packages/node/wit", world: "node" });

use crate::types::{RepositoryOutput, RepositoryProperties};
use giong_node::types::metadata::Metadata;
use giong_node::types::node::OUTPUT_KEY;
use giong_node::types::tools::git::{CheckoutInputs, CheckoutOutput};
use giong_node::types::tools::secrets::SecretValue;

struct Repository;

impl Guest for Repository {
    fn metadata() -> Result<String, String> {
        serde_json::to_string(&Metadata {
            kind: "sources/repository".into(),
            name: "Repository".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: "Check out a Git revision into a host-provided workspace".into(),
        })
        .map_err(|error| error.to_string())
    }

    fn configure(args: String) -> Result<(), String> {
        let inputs: RepositoryProperties =
            serde_json::from_str(&args).map_err(|_| "Invalid checkout properties".to_string())?;
        let token = inputs
            .token
            .map(|token| match token {
                SecretValue::Literal(value) => Ok(value),
                SecretValue::Reference { secret } => giong::node::secrets::get(&secret),
            })
            .transpose()?;
        let request =
            CheckoutInputs { repo_url: inputs.repo_url, revision: inputs.revision, token };
        let output = giong::node::git::checkout(
            &serde_json::to_string(&request).map_err(|_| "Invalid checkout inputs")?,
        )?;
        let output: CheckoutOutput =
            serde_json::from_str(&output).map_err(|_| "Invalid checkout output")?;
        let previous = giong::node::storage::get("repository:commit-sha")?;
        let bytes =
            serde_json::to_vec(&RepositoryOutput { repository_path: output.repository_path })
                .map_err(|_| "Could not encode repository output")?;
        giong::node::storage::put(OUTPUT_KEY, &bytes)?;
        if previous.as_deref() != Some(output.commit_sha.as_bytes()) {
            giong::node::events::emit("changed")?;
            giong::node::storage::put("repository:commit-sha", output.commit_sha.as_bytes())?;
        }
        Ok(())
    }
}
export!(Repository);
