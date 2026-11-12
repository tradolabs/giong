wit_bindgen::generate!({ path: "../../../packages/node/wit", world: "node" });

use giong_node::types::metadata::Metadata;
use giong_node::types::node::OUTPUT_KEY;
use giong_node::types::tools::execution::Request;
use giong_node::types::tools::git::ChangedFilesInputs;
use giong_node::types::tools::secrets::SecretValue;
use giong_repository_source::types::RepositoryOutput;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Inputs {
    source: String,
    path: Option<String>,
    values_file: Option<String>,
    namespace: Option<String>,
    release: Option<String>,
    kubeconfig: Option<SecretValue>,
    #[serde(default)]
    env: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize)]
struct DeploymentState {
    revision: String,
    configuration: String,
}

#[derive(Serialize)]
struct KubectlOutput {
    exit_code: i32,
    skipped: bool,
    revision: String,
}

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

    fn configure(args: String) -> Result<(), String> {
        let inputs: Inputs = serde_json::from_str(&args).map_err(|_| "Invalid kubectl inputs")?;
        let source = giong::node::nodes::output(&inputs.source)?;
        let source: RepositoryOutput =
            serde_json::from_str(&source).map_err(|_| "Invalid source node output")?;
        let commit_sha = giong::node::git::head(&source.repository_path)?;
        let path = inputs.path.as_deref().unwrap_or(".");
        if std::path::Path::new(path).components().any(|part| {
            !matches!(part, std::path::Component::Normal(_) | std::path::Component::CurDir)
        }) {
            return Err("Chart path must be relative without parent traversal".into());
        }
        let chart = std::path::Path::new(&source.repository_path).join(path);
        let relative_path: std::path::PathBuf = chart
            .strip_prefix(&source.repository_path)
            .map_err(|_| "Source path is outside its repository")?
            .components()
            .filter(|part| *part != std::path::Component::CurDir)
            .collect();
        let relative_path = if relative_path.as_os_str().is_empty() {
            ".".into()
        } else {
            relative_path.to_str().ok_or("Chart path is not UTF-8")?.to_owned()
        };
        let chart = chart.to_str().ok_or("Chart path is not UTF-8")?.to_owned();
        let mut env = inputs.env;
        if let Some(kubeconfig) = inputs.kubeconfig {
            let value = match kubeconfig {
                SecretValue::Literal(value) => value,
                SecretValue::Reference { secret } => giong::node::secrets::get(&secret)?,
            };
            env.insert("KUBECONFIG".into(), value);
        }
        let release = inputs.release.unwrap_or_else(|| "release".into());
        let namespace = inputs.namespace.unwrap_or_else(|| "default".into());
        let values_file = inputs.values_file.unwrap_or_default();
        let paths = comparison_paths(&relative_path, &values_file)?;
        // Runtime checkout paths change every run; fingerprint only stable deployment inputs.
        // Include resolved credentials so credential rotation also triggers deployment.
        let configuration = serde_json::to_vec(&json!({
            "source": inputs.source, "path": relative_path,
            "release": release, "namespace": namespace, "values_file": values_file,
            "env": env, "dockerfile": include_str!("../Dockerfile"),
            "version": env!("CARGO_PKG_VERSION"),
        }))
        .map_err(|_| "Could not fingerprint deployment configuration")?;
        let configuration = format!("{:x}", Sha256::digest(configuration));
        let scope = giong::node::nodes::scope()?;
        let state_key = format!("kubectl:last-successful-deployment:{scope}");
        let previous = giong::node::storage::get(&state_key)?
            .map(|bytes| {
                serde_json::from_slice::<DeploymentState>(&bytes)
                    .map_err(|_| "Invalid saved deployment state".to_string())
            })
            .transpose()?;
        if let Some(previous) = previous.filter(|previous| previous.configuration == configuration)
        {
            let unchanged = if previous.revision == commit_sha {
                true
            } else {
                let comparison = ChangedFilesInputs {
                    repository_path: source.repository_path.clone(),
                    from_revision: previous.revision,
                    to_revision: commit_sha.clone(),
                    paths,
                };
                let request =
                    serde_json::to_string(&comparison).map_err(|_| "Invalid comparison request")?;
                // A removed/rewritten base commit must cause deployment rather than a false skip.
                giong::node::git::changed_files(&request)
                    .ok()
                    .and_then(|output| serde_json::from_str::<Vec<String>>(&output).ok())
                    .is_some_and(|files| files.is_empty())
            };
            if unchanged {
                return save_output(KubectlOutput {
                    exit_code: 0,
                    skipped: true,
                    revision: commit_sha,
                });
            }
        }
        let request = Request {
            dockerfile: include_str!("../Dockerfile").into(),
            dockerignore: Some("**/.git\n".into()),
            env,
            args: vec![release, namespace, values_file, chart],
        };
        let result = giong::node::execution::execute(
            &serde_json::to_string(&request).map_err(|_| "Invalid execution request")?,
        )?;
        let output: serde_json::Value =
            serde_json::from_str(&result).map_err(|_| "Invalid execution result")?;
        if output.get("exit_code").and_then(serde_json::Value::as_i64) != Some(0) {
            return Err("Deployment did not succeed".into());
        }
        let state = DeploymentState { revision: commit_sha.clone(), configuration };
        giong::node::storage::put(
            &state_key,
            &serde_json::to_vec(&state).map_err(|_| "Invalid deployment state")?,
        )?;
        save_output(KubectlOutput { exit_code: 0, skipped: false, revision: commit_sha })
    }
}

fn save_output(output: KubectlOutput) -> Result<(), String> {
    let bytes = serde_json::to_vec(&output).map_err(|_| "Could not encode kubectl output")?;
    giong::node::storage::put(OUTPUT_KEY, &bytes)
}

// Values files may live outside the chart directory, but must stay in the source repository.
fn comparison_paths(chart: &str, values_file: &str) -> Result<Vec<String>, String> {
    let mut paths = vec![chart.to_owned()];
    if !values_file.is_empty() {
        let mut path = std::path::PathBuf::new();
        for component in std::path::Path::new(chart).join(values_file).components() {
            match component {
                std::path::Component::CurDir => {}
                std::path::Component::Normal(part) => path.push(part),
                std::path::Component::ParentDir if path.pop() => {}
                _ => return Err("Values file must stay within the source repository".into()),
            }
        }
        paths.push(path.to_str().ok_or("Values file path is not UTF-8")?.to_owned());
    }
    Ok(paths)
}

export!(Kubectl);
