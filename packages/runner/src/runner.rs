use crate::error::Error;
use crate::plugins;
use crate::types::{Plan, Secret};
use crate::utils::process::run_command;
use serde_json::Value;
use std::{
    fs::{self, File},
    path::Path,
    process::Command,
};

pub fn run(
    job_id: &str,
    stack_id: &str,
    stack_name: &str,
    configurations: &Value,
    secrets: &[Secret],
    working_dir: &Path,
    logger: &mut File,
) -> Result<Option<i32>, Error> {
    let plan = plugins::plan(stack_name, configurations)?;
    match &plan {
        Plan::Docker { dockerfile, dockerignore, args } => {
            let image = format!("giong-image:{job_id}");
            let code =
                build_image(&image, dockerfile, dockerignore.as_deref(), working_dir, logger)?;
            if code != Some(0) {
                return Ok(code);
            }

            let result = run_container(&image, stack_id, args, secrets, logger);
            let _ = run_command(Command::new("docker").args(["image", "rm", &image]), logger);
            result
        }
    }
}

fn build_image(
    image: &str,
    dockerfile_content: &str,
    dockerignore_content: Option<&str>,
    working_dir: &Path,
    logger: &mut File,
) -> Result<Option<i32>, Error> {
    let context = fs::canonicalize(working_dir)?;
    if !context.is_dir() {
        return Err(Error::Unexpected(
            "Stack folder must be a directory within the repository checkout".into(),
        ));
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
    stack_id: &str,
    args: &[String],
    secrets: &[Secret],
    logger: &mut File,
) -> Result<Option<i32>, Error> {
    let mut command = Command::new("docker");
    command.args(["run", "--rm"]).args(["--add-host", "host.docker.internal:host-gateway"]);

    for secret in secrets {
        command.arg("--env").arg(&secret.key).env(&secret.key, &secret.value);
    }

    run_command(
        command.arg("--env").arg(format!("GIONG_STACK_ID={stack_id}")).arg(image).args(args),
        logger,
    )
}
