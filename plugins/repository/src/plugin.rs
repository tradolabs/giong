use anyhow::{Context as _, Result};
use giong_node::sdk::{Context, Plugin};
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
        ctx: &mut Context,
        RepositoryInput { repo_url, revision, token }: RepositoryInput,
    ) -> Result<()> {
        let workspace = ctx.open_workspace()?;
        let token = token.map(|token| ctx.resolve_secret(token)).transpose()?;
        let token = token.as_deref();

        ctx.log.info("Resolving repository revision...")?;
        let revision = giong_git::resolve_revision(&repo_url, &revision, token)
            .context("Could not resolve repository revision")?;
        let repository = workspace.join(REPOSITORY_DIR);
        let repository_path = repository.to_str().context("Repository path is not UTF-8")?;
        if repository.exists() {
            fs::remove_dir_all(&repository)?;
        }

        ctx.log.info(format!("Cloning repository at {revision}..."))?;
        giong_git::checkout(&repo_url, &repository, &revision, token)
            .inspect_err(|_| {
                let _ = fs::remove_dir_all(&repository);
            })
            .context("Could not check out repository")?;

        let previous = ctx.get_storage(COMMIT_SHA_KEY)?;
        ctx.save_output(&RepositoryOutput { repository_path: repository_path.into() })?;
        if previous.as_deref() == Some(revision.as_bytes()) {
            ctx.log.info("No new commits detected.")?;
        } else {
            ctx.log.info("New commit detected.")?;
            ctx.put_storage(COMMIT_SHA_KEY, revision.into_bytes())?;
        }
        ctx.emit_changed()
    }
}
