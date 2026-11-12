use crate::runtime::{Host, bindings};
use crate::types::tools::git::{ChangedFilesInputs, CheckoutInputs, CheckoutOutput};
use anyhow::{Context, bail};
use std::fs;

impl bindings::giong::node::git::Host for Host {
    fn checkout(&mut self, inputs: String) -> Result<String, String> {
        let result = (|| -> anyhow::Result<String> {
            let root = self
                .workspace
                .as_ref()
                .context("Repository access is not granted to this plugin")?;
            let inputs: CheckoutInputs = serde_json::from_str(&inputs)?;
            if inputs.repo_url.trim().is_empty() || inputs.revision.trim().is_empty() {
                bail!("Repository URL and revision are required");
            }
            let revision = giong_git::resolve_revision(
                &inputs.repo_url,
                &inputs.revision,
                inputs.token.as_deref(),
            )?;
            let parent = root.parent().context("Invalid node workspace")?;
            fs::create_dir_all(parent)?;
            let staging = tempfile::tempdir_in(parent)?;
            let checkout = staging.path().join("repository");
            giong_git::checkout(&inputs.repo_url, &checkout, &revision, inputs.token.as_deref())?;
            if root.exists() {
                fs::remove_dir_all(root)?;
            }
            fs::rename(checkout, root)?;
            let root = fs::canonicalize(root)?;
            Ok(serde_json::to_string(&CheckoutOutput {
                repository_path: root.to_str().context("Repository path is not UTF-8")?.into(),
                commit_sha: revision,
            })?)
        })();
        // Avoid returning Git errors that may include credentials or authenticated URLs.
        result.map_err(|_| {
            "Repository checkout failed; check access, revision, path and workspace".into()
        })
    }

    fn head(&mut self, repository_path: String) -> Result<String, String> {
        let result = (|| -> anyhow::Result<String> {
            let workspace = self.workspace.as_ref().context("Git access is not granted")?;
            let allowed = fs::canonicalize(workspace.parent().context("Invalid workspace")?)?;
            let repository = fs::canonicalize(repository_path)?;
            if !repository.starts_with(&allowed)
                || !fs::canonicalize(repository.join(".git"))?.starts_with(&allowed)
            {
                bail!("Repository must belong to this flow workspace");
            }
            Ok(giong_git::head(&repository)?)
        })();
        result.map_err(|_| "Could not read local Git HEAD".into())
    }

    fn changed_files(&mut self, inputs: String) -> Result<String, String> {
        let result = (|| -> anyhow::Result<String> {
            let workspace = self.workspace.as_ref().context("Git access is not granted")?;
            let allowed = fs::canonicalize(workspace.parent().context("Invalid workspace")?)?;
            let inputs: ChangedFilesInputs = serde_json::from_str(&inputs)?;
            let repository = fs::canonicalize(&inputs.repository_path)?;
            if !repository.starts_with(&allowed)
                || !fs::canonicalize(repository.join(".git"))?.starts_with(&allowed)
            {
                bail!("Repository must belong to this flow workspace");
            }
            for path in &inputs.paths {
                if std::path::Path::new(path).components().any(|part| {
                    !matches!(part, std::path::Component::Normal(_) | std::path::Component::CurDir)
                }) {
                    bail!("Comparison paths must be relative without parent traversal");
                }
            }
            let files = giong_git::changed_files(
                &repository,
                &inputs.from_revision,
                &inputs.to_revision,
                &inputs.paths,
            )?;
            Ok(serde_json::to_string(&files)?)
        })();
        result.map_err(|_| "Could not compare local Git revisions".into())
    }
}
