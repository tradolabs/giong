use crate::configuration::DeploymentConfiguration;
use crate::giong::node::{execution, git, nodes, storage};
use crate::types::{DeploymentState, KubectlOutput, KubectlProperties};
use giong_node::types::node::OUTPUT_KEY;
use giong_node::types::tools::execution::Request;
use giong_node::types::tools::git::ChangedFilesInputs;
use giong_repository_source::types::RepositoryOutput;

pub(crate) fn execute(args: &str) -> Result<(), String> {
    let properties: KubectlProperties =
        serde_json::from_str(args).map_err(|_| "Invalid kubectl inputs")?;
    println!("Preparing kubectl deployment...");

    let source = read_source(&properties.source)?;
    let revision = git::head(&source.repository_path)?;
    println!("Source commit: {revision}");

    let configuration = DeploymentConfiguration::new(properties, &source.repository_path)?;
    let paths = configuration.comparison_paths()?;
    let current = DeploymentState { revision, configuration: configuration.fingerprint()? };
    let state_key = format!("kubectl:last-successful-deployment:{}", nodes::scope()?);
    let previous = read_state(&state_key)?;

    if should_skip(previous.as_ref(), &current, &source.repository_path, paths)? {
        return save_output(current.revision, true);
    }

    deploy(configuration.into_request())?;
    save_state(&state_key, &current)?;
    save_output(current.revision, false)?;
    println!("Deployment completed successfully.");
    Ok(())
}

fn read_source(name: &str) -> Result<RepositoryOutput, String> {
    let output = nodes::output(name)?;
    serde_json::from_str(&output).map_err(|_| "Invalid source node output".into())
}

fn should_skip(
    previous: Option<&DeploymentState>,
    current: &DeploymentState,
    repository_path: &str,
    paths: Vec<String>,
) -> Result<bool, String> {
    let Some(previous) = previous else {
        println!("No previous successful deployment; deployment required.");
        return Ok(false);
    };
    if previous.configuration != current.configuration {
        println!("Deployment configuration changed; deployment required.");
        return Ok(false);
    }
    if previous.revision == current.revision {
        println!(
            "Skipping deployment: this commit and configuration were already deployed successfully."
        );
        return Ok(true);
    }

    println!("Checking chart and values files for changes...");
    let comparison = ChangedFilesInputs {
        repository_path: repository_path.to_owned(),
        from_revision: previous.revision.clone(),
        to_revision: current.revision.clone(),
        paths,
    };
    let request = serde_json::to_string(&comparison).map_err(|_| "Invalid comparison request")?;
    // A missing or rewritten base commit requires deployment, rather than a false skip.
    let unchanged = git::changed_files(&request)
        .ok()
        .and_then(|output| serde_json::from_str::<Vec<String>>(&output).ok())
        .is_some_and(|files| files.is_empty());
    if unchanged {
        println!(
            "Skipping deployment: chart and values files are unchanged since the last successful deployment."
        );
    } else {
        println!("Relevant files changed or comparison unavailable; deployment required.");
    }
    Ok(unchanged)
}

fn deploy(request: Request) -> Result<(), String> {
    println!("Building deployment image and running Helm and kubectl...");
    let request = serde_json::to_string(&request).map_err(|_| "Invalid execution request")?;
    let result = execution::execute(&request)?;
    let output: serde_json::Value =
        serde_json::from_str(&result).map_err(|_| "Invalid execution result")?;
    if output.get("exit_code").and_then(serde_json::Value::as_i64) != Some(0) {
        return Err("Deployment did not succeed".into());
    }
    Ok(())
}

fn read_state(key: &str) -> Result<Option<DeploymentState>, String> {
    storage::get(key)?
        .map(|bytes| {
            serde_json::from_slice(&bytes).map_err(|_| "Invalid saved deployment state".into())
        })
        .transpose()
}

fn save_state(key: &str, state: &DeploymentState) -> Result<(), String> {
    let bytes = serde_json::to_vec(state).map_err(|_| "Invalid deployment state")?;
    storage::put(key, &bytes)
}

fn save_output(revision: String, skipped: bool) -> Result<(), String> {
    let output = KubectlOutput { exit_code: 0, skipped, revision };
    let bytes = serde_json::to_vec(&output).map_err(|_| "Could not encode kubectl output")?;
    storage::put(OUTPUT_KEY, &bytes)
}
