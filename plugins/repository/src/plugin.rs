use anyhow::{Context, Result};
use giong_node::sdk::{Client, Plugin};
use giong_node::types::metadata::Metadata;
use giong_plugins_repository::{RepositoryInput, RepositoryOutput};
use std::fs;

const COMMIT_SHA_KEY: &str = "repository:commit-sha";
const REPOSITORY_DIR: &str = "repository";

pub(crate) struct Repository;

impl Plugin for Repository {
    type Input = RepositoryInput;

    fn metadata() -> Metadata {
        Metadata {
            kind: "repository".into(),
            name: "Repository".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: "Check out a Git revision into the node workspace".into(),
        }
    }

    fn execute(
        client: &mut Client<'_>,
        RepositoryInput { repo_url, revision, token }: RepositoryInput,
    ) -> Result<()> {
        let workspace = client.open_workspace()?;
        let token = token.map(|token| client.resolve_secret(token)).transpose()?;
        let token = token.as_deref();

        eprintln!("Resolving repository revision...");
        let revision = giong_git::resolve_revision(&repo_url, &revision, token)
            .context("Could not resolve repository revision")?;
        let repository = workspace.join(REPOSITORY_DIR);
        let repository_path = repository.to_str().context("Repository path is not UTF-8")?;
        if repository.exists() {
            fs::remove_dir_all(&repository)?;
        }

        eprintln!("Cloning repository at {revision}...");
        giong_git::checkout(&repo_url, &repository, &revision, token)
            .inspect_err(|_| {
                let _ = fs::remove_dir_all(&repository);
            })
            .context("Could not check out repository")?;

        let previous = client.get_storage(COMMIT_SHA_KEY)?;
        client.save_output(&RepositoryOutput { repository_path: repository_path.into() })?;
        if previous.as_deref() == Some(revision.as_bytes()) {
            eprintln!("No new commits detected.");
        } else {
            eprintln!("New commit detected.");
            client.put_storage(COMMIT_SHA_KEY, revision.into_bytes())?;
        }
        client.emit_changed()
    }
}
