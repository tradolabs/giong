use crate::config::Config;
use crate::error::Error;
use include_dir::include_dir;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    process::Command,
};

use crate::utils::filesystem::copy_dir;
use crate::utils::process::run_command;

pub fn run_docker(
    image: &str,
    stack_id: &str,
    credentials: &[(String, String)],
    working_dir: &Path,
    logger: &mut File,
) -> Result<Option<i32>, Error> {
    let config_path = working_dir.join("giong.toml");
    let content = fs::read_to_string(&config_path)
        .map_err(|source| Error::ConfigRead { path: config_path, source })?;
    let config: Config = toml::from_str(&content)?;
    let dockerfile = config.toolchain.language.dockerfile();
    let dir = include_dir!("$CARGO_MANIFEST_DIR/dockerfiles")
        .get_dir(dockerfile)
        .ok_or_else(|| Error::DockerfileNotFound(dockerfile.to_owned()))?;

    copy_dir(dir, working_dir).map_err(|error| Error::TemplateCopy(Box::new(error)))?;

    let code = run_command(
        Command::new("docker").args(["build", "--tag", image, "."]).current_dir(working_dir),
        logger,
    )?;
    if code != Some(0) {
        return Ok(code);
    }

    let mut environment = tempfile::NamedTempFile::new()?;
    for (key, value) in credentials {
        writeln!(environment, "{key}={value}")?;
    }
    environment.flush()?;

    let result = run_command(
        Command::new("docker").args(["run", "--rm", "--env-file"]).arg(environment.path()).args([
            "--add-host",
            "host.docker.internal:host-gateway",
            "--env",
            "GIONG_RECONCILER_URL=http://host.docker.internal:2929",
            "--env",
            &format!("GIONG_STACK_ID={stack_id}"),
            image,
        ]),
        logger,
    );
    let _ = run_command(Command::new("docker").args(["image", "rm", image]), logger);
    result
}
