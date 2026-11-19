use crate::types::{DeploymentState, KubectlOutput, KubectlProperties};
use anyhow::{Context, Result, bail, ensure};
use giong_node::sdk::{Client, Plugin};
use giong_node::types::metadata::Metadata;
use giong_plugins_repository::RepositoryOutput;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

pub(crate) struct Kubectl;

impl Plugin for Kubectl {
    type Input = KubectlProperties;

    fn metadata() -> Metadata {
        Metadata {
            kind: "kubectl".into(),
            name: "kubectl".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            description: "Render a Helm chart from a source node and apply it with kubectl".into(),
        }
    }

    fn execute(client: &mut Client<'_>, properties: KubectlProperties) -> Result<()> {
        eprintln!("Preparing kubectl deployment...");

        let source: RepositoryOutput = client.get_output(&properties.source)?;
        let workspace = client.get_source_workspace(&properties.source)?;
        let repository = giong_git::workspace_checkout(
            Path::new(&workspace),
            Path::new(&source.repository_path),
        )?;
        let revision = giong_git::head(&repository)?;
        eprintln!("Source commit: {revision}");

        let configuration = DeploymentConfiguration::new(client, properties)?;
        let paths = configuration.comparison_paths()?;
        let current = DeploymentState { revision, configuration: configuration.fingerprint()? };
        let state_key = format!("kubectl:last-successful-deployment:{}", client.get_scope()?);
        let previous = read_state(client, &state_key)?;

        if should_skip(previous.as_ref(), &current, &repository, &paths)? {
            return save_output(client, current.revision, true);
        }

        configuration.deploy(&repository, &client.open_workspace()?)?;
        save_state(client, &state_key, &current)?;
        save_output(client, current.revision, false)?;
        eprintln!("Deployment completed successfully.");
        Ok(())
    }
}

fn should_skip(
    previous: Option<&DeploymentState>,
    current: &DeploymentState,
    repository: &Path,
    paths: &[String],
) -> Result<bool> {
    let Some(previous) = previous else {
        eprintln!("No previous successful deployment; deployment required.");
        return Ok(false);
    };
    if previous.configuration != current.configuration {
        eprintln!("Deployment configuration changed; deployment required.");
        return Ok(false);
    }
    if previous.revision == current.revision {
        eprintln!(
            "Skipping deployment: this commit and configuration were already deployed successfully."
        );
        return Ok(true);
    }

    eprintln!("Checking chart and values files for changes...");
    // A missing or rewritten base commit requires deployment.
    let unchanged =
        giong_git::changed_files(repository, &previous.revision, &current.revision, paths)
            .is_ok_and(|files| files.is_empty());
    if unchanged {
        eprintln!(
            "Skipping deployment: chart and values files are unchanged since the last successful deployment."
        );
    } else {
        eprintln!("Relevant files changed or comparison unavailable; deployment required.");
    }
    Ok(unchanged)
}

fn read_state(client: &mut Client<'_>, key: &str) -> Result<Option<DeploymentState>> {
    client
        .get_storage(key)?
        .map(|bytes| serde_json::from_slice(&bytes).context("Invalid saved deployment state"))
        .transpose()
}

fn save_state(client: &mut Client<'_>, key: &str, state: &DeploymentState) -> Result<()> {
    let bytes = serde_json::to_vec(state).context("Invalid deployment state")?;
    client.put_storage(key, bytes)
}

fn save_output(client: &mut Client<'_>, revision: String, skipped: bool) -> Result<()> {
    let output = KubectlOutput { exit_code: 0, skipped, revision };
    client.save_output(&output)
}

struct DeploymentConfiguration {
    source: String,
    relative_chart: String,
    release: String,
    namespace: String,
    values_file: String,
    env: BTreeMap<String, String>,
}

impl DeploymentConfiguration {
    fn new(client: &mut Client<'_>, properties: KubectlProperties) -> Result<Self> {
        let path = Path::new(properties.path.as_deref().unwrap_or("."));
        if path.components().any(|part| !matches!(part, Component::Normal(_) | Component::CurDir)) {
            bail!("Chart path must be relative without parent traversal");
        }
        let relative_chart: PathBuf =
            path.components().filter(|part| *part != Component::CurDir).collect();
        let relative_chart = if relative_chart.as_os_str().is_empty() {
            ".".into()
        } else {
            relative_chart.to_str().context("Chart path is not UTF-8")?.to_owned()
        };

        let mut env = properties.env;
        if let Some(kubeconfig) = properties.kubeconfig {
            let value = client.resolve_secret(kubeconfig)?;
            env.insert("KUBECONFIG".into(), value);
        }

        Ok(Self {
            source: properties.source,
            relative_chart,
            release: properties.release.unwrap_or_else(|| "release".into()),
            namespace: properties.namespace.unwrap_or_else(|| "default".into()),
            values_file: properties.values_file.unwrap_or_default(),
            env,
        })
    }

    fn fingerprint(&self) -> Result<String> {
        // Keep checkout locations out of the fingerprint: they change between runs.
        // Resolved credentials are included so rotation triggers a new deployment.
        let bytes = serde_json::to_vec(&json!({
            "source": self.source,
            "path": self.relative_chart,
            "release": self.release,
            "namespace": self.namespace,
            "values_file": self.values_file,
            "env": self.env,
            "dockerfile": include_str!("../Dockerfile"),
            "version": env!("CARGO_PKG_VERSION"),
        }))
        .context("Could not fingerprint deployment configuration")?;
        Ok(format!("{:x}", Sha256::digest(bytes)))
    }

    fn comparison_paths(&self) -> Result<Vec<String>> {
        let mut paths = vec![self.relative_chart.clone()];
        if self.values_file.is_empty() {
            return Ok(paths);
        }

        // Values files may be outside the chart, but must stay inside the repository.
        let mut path = PathBuf::new();
        for component in Path::new(&self.relative_chart).join(&self.values_file).components() {
            match component {
                Component::CurDir => {}
                Component::Normal(part) => path.push(part),
                Component::ParentDir if path.pop() => {}
                _ => bail!("Values file must stay within the source repository"),
            }
        }
        paths.push(path.to_str().context("Values file path is not UTF-8")?.to_owned());
        Ok(paths)
    }

    fn deploy(mut self, repository: &Path, workspace: &Path) -> Result<()> {
        let temporary = tempfile::tempdir_in(workspace)?;
        let copy = temporary.path().join("repository");
        fs::create_dir(&copy)?;
        run_logged(Command::new("cp").arg("-R").arg(repository.join(".")).arg(&copy))?;
        let chart = copy.join(&self.relative_chart).canonicalize()?;
        ensure!(chart.starts_with(&copy), "Chart must stay within the repository");

        let kubeconfig = self.env.remove("KUBECONFIG").context("Kubeconfig is required")?;
        let mut config = tempfile::NamedTempFile::new_in(temporary.path())?;
        config.write_all(kubeconfig.as_bytes())?;
        self.env.insert("HOME".into(), temporary.path().to_string_lossy().into_owned());
        self.env.insert("KUBECONFIG".into(), config.path().to_string_lossy().into_owned());

        eprintln!("Building Helm dependencies...");
        run_logged(Command::new("helm").args(["dependency", "build"]).arg(&chart).envs(&self.env))?;
        let manifests = temporary.path().join("manifests.yaml");
        let output = File::create(&manifests)?;
        let mut helm = Command::new("helm");
        helm.arg("template")
            .arg(&self.release)
            .arg(&chart)
            .args(["--namespace", &self.namespace, "--no-hooks"])
            .envs(&self.env)
            .stdin(Stdio::null())
            .stdout(output);
        if !self.values_file.is_empty() {
            let values = chart.join(&self.values_file).canonicalize()?;
            ensure!(values.starts_with(&copy), "Values file must stay within the repository");
            helm.arg("--values").arg(values);
        }
        ensure!(helm.status()?.success(), "Helm template failed");

        eprintln!("Applying manifests with kubectl...");
        run_logged(
            Command::new("kubectl")
                .args(["--namespace", &self.namespace, "apply", "-f"])
                .arg(manifests)
                .envs(&self.env),
        )
    }
}

// stdout is reserved for the plugin protocol.
fn run_logged(command: &mut Command) -> Result<()> {
    let mut process = command.stdin(Stdio::null()).stdout(Stdio::piped()).spawn()?;
    let mut stdout = process.stdout.take().expect("command stdout is piped");
    let copied = io::copy(&mut stdout, &mut io::stderr().lock());
    drop(stdout);
    let status = process.wait()?;
    copied?;
    ensure!(status.success(), "{} exited with {status}", command.get_program().to_string_lossy());
    Ok(())
}
