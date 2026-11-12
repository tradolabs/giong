use crate::runtime::{Host, bindings};
use crate::types::tools::execution::Request;
use anyhow::{Context, ensure};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub struct Execution {
    pub id: String,
    pub workspace: PathBuf,
    pub logger: File,
}

impl bindings::giong::node::execution::Host for Host {
    fn execute(&mut self, request: String) -> Result<String, String> {
        let result = (|| -> anyhow::Result<String> {
            let execution = self.execution.as_mut().context("Execution access is not granted")?;
            let request: Request = serde_json::from_str(&request)?;
            let code = build_image(
                &format!("giong-image:{}", execution.id),
                &request.dockerfile,
                request.dockerignore.as_deref(),
                &execution.workspace,
                &mut execution.logger,
            )?;
            ensure!(code == Some(0), "Container build failed");
            let image = format!("giong-image:{}", execution.id);
            let result = run_container(
                &image,
                &execution.id,
                &request.args,
                &request.env,
                &execution.workspace,
                &mut execution.logger,
            );
            let _ = run_command(
                Command::new("docker").args(["image", "rm", &image]),
                &mut execution.logger,
            );
            let exit_code = result?;
            ensure!(exit_code == Some(0), "Container execution failed");
            Ok(serde_json::json!({ "exit_code": exit_code }).to_string())
        })();
        result.map_err(|_| "Execution failed; see the run log".into())
    }
}

fn build_image(
    image: &str,
    dockerfile_content: &str,
    dockerignore_content: Option<&str>,
    working_dir: &Path,
    logger: &mut File,
) -> anyhow::Result<Option<i32>> {
    let context = fs::canonicalize(working_dir)?;
    if !context.is_dir() {
        anyhow::bail!("Working directory must be a directory");
    }
    let build_files = tempfile::tempdir()?;
    let dockerfile = build_files.path().join("Dockerfile");
    fs::write(&dockerfile, dockerfile_content)?;
    if let Some(ignore) = dockerignore_content {
        fs::write(build_files.path().join("Dockerfile.dockerignore"), ignore)?;
    }
    let mut command = Command::new("docker");
    command.args(["build", "--tag", image, "--file"]).arg(&dockerfile);
    run_command(command.arg(".").current_dir(&context), logger)
}

fn run_container(
    image: &str,
    flow_id: &str,
    args: &[String],
    env: &std::collections::BTreeMap<String, String>,
    working_dir: &Path,
    logger: &mut File,
) -> anyhow::Result<Option<i32>> {
    let mut command = Command::new("docker");
    command.args(["run", "--rm"]).args(["--add-host", "host.docker.internal:host-gateway"]);

    for (key, value) in env {
        command.arg("--env").arg(key).env(key, value);
    }

    let context = fs::canonicalize(working_dir)?;
    let args: Vec<String> = args
        .iter()
        .map(|argument| match Path::new(argument).strip_prefix(&context) {
            Ok(relative) => format!("/workspace/{}", relative.display()),
            Err(_) => argument.clone(),
        })
        .collect();
    run_command(
        command.arg("--env").arg(format!("GIONG_FLOW_ID={flow_id}")).arg(image).args(args),
        logger,
    )
}

fn run_command(command: &mut Command, log_path: &mut File) -> anyhow::Result<Option<i32>> {
    writeln!(
        log_path,
        "\n$ {:?} {:?}",
        command.get_program(),
        command.get_args().collect::<Vec<_>>()
    )?;
    log_path.flush()?;
    let status = command
        .stdin(Stdio::null())
        .stdout(log_path.try_clone()?)
        .stderr(log_path.try_clone()?)
        .status()?;
    Ok(status.code())
}
