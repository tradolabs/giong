wit_bindgen::generate!({ path: "../../packages/node/wit", world: "node" });

use self::giong::node::{events, git, secrets, storage};
use crate::types::{RepositoryOutput, RepositoryProperties};
use giong_node::types::metadata::Metadata;
use giong_node::types::node::OUTPUT_KEY;
use giong_node::types::tools::git::{CheckoutInputs, CheckoutOutput};
use giong_node::types::tools::secrets::SecretValue;

const COMMIT_SHA_KEY: &str = "repository:commit-sha";

struct Repository;

impl Guest for Repository {
    fn metadata() -> Result<String, String> {
        serde_json::to_string(&Metadata {
            kind: "repository".into(),
            name: "Repository".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: "Check out a Git revision into a host-provided workspace".into(),
        })
        .map_err(|error| error.to_string())
    }

    fn execute(args: String) -> Result<(), String> {
        let properties: RepositoryProperties =
            serde_json::from_str(&args).map_err(|_| "Invalid checkout properties".to_string())?;
        println!("Preparing repository checkout...");

        let output = checkout(properties)?;
        let previous = storage::get(COMMIT_SHA_KEY)?;
        save_output(output.repository_path)?;
        publish_change(&output.commit_sha, previous.as_deref())
    }
}

fn checkout(properties: RepositoryProperties) -> Result<CheckoutOutput, String> {
    let token = properties
        .token
        .map(|token| match token {
            SecretValue::Literal(value) => Ok(value),
            SecretValue::Reference { secret } => secrets::get(&secret),
        })
        .transpose()?;
    let request =
        CheckoutInputs { repo_url: properties.repo_url, revision: properties.revision, token };
    let request = serde_json::to_string(&request).map_err(|_| "Invalid checkout inputs")?;
    let output = git::checkout(&request)?;
    serde_json::from_str(&output).map_err(|_| "Invalid checkout output".into())
}

fn save_output(repository_path: String) -> Result<(), String> {
    let output = RepositoryOutput { repository_path };
    let bytes = serde_json::to_vec(&output).map_err(|_| "Could not encode repository output")?;
    storage::put(OUTPUT_KEY, &bytes)?;
    println!("Repository output saved.");
    Ok(())
}

fn publish_change(commit_sha: &str, previous: Option<&[u8]>) -> Result<(), String> {
    if previous == Some(commit_sha.as_bytes()) {
        println!("No new commits detected.");
        return Ok(());
    }

    println!("New commit detected; triggering dependent nodes...");
    events::emit("changed")?;
    storage::put(COMMIT_SHA_KEY, commit_sha.as_bytes())
}

export!(Repository);
