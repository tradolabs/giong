use crate::{
    config::Config,
    credentials::Credentials,
    utils::{filesystem::copy_dir, process::run_command},
};
use anyhow::{Context, Result};
use giong_database::{JobEntity, JobStatus, RepositoryEntity, StackEntity};
use include_dir::include_dir;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

pub struct Job {
    root_folder: PathBuf,
    repository: RepositoryEntity,
    stack: StackEntity,
    pub(crate) job: JobEntity,
}

impl Job {
    pub fn new(
        root_folder: PathBuf,
        repository: RepositoryEntity,
        stack: StackEntity,
        job: JobEntity,
    ) -> Self {
        Self { root_folder, repository, stack, job }
    }

    pub async fn run(self) -> (JobStatus, Option<i32>) {
        let job_id = self.job.id;

        let result = tokio::task::spawn_blocking(move || -> Result<Option<i32>> {
            let log_folder = self.root_folder.join("logs");
            let mut logger = File::create(log_folder.join(self.job.id.to_string()))?;
            let result = (|| {
                let (working_dir, config) = self.checkout()?;
                let language = &config.toolchain.language;
                let image = format!("giong-image:{}", self.job.id);
                let dockerfile = language.dockerfile();
                Self::run_container(&image, &working_dir, dockerfile, &mut logger)
            })();
            if let Err(error) = &result {
                let _ = writeln!(logger, "{error:#}");
            }
            result
        })
        .await
        .unwrap_or_else(|error| Err(error.into()));

        match result {
            Ok(Some(0)) => (JobStatus::Succeeded, Some(0)),
            Ok(code) => (JobStatus::Failed, code),
            Err(error) => {
                eprintln!("Job {job_id}: {error:#}");
                (JobStatus::Failed, None)
            }
        }
    }

    fn checkout(&self) -> Result<(PathBuf, Config)> {
        let checkout_path = self.root_folder.join("runs").join(self.job.id.to_string());
        let stack_path = checkout_path.join(&self.stack.folder_path);
        let token = self
            .repository
            .credentials
            .as_deref()
            .map(|value| Credentials::new(self.repository.url.as_bytes()).decrypt(value))
            .transpose()?;
        giong_git::checkout(
            &self.repository.url,
            &checkout_path,
            &self.job.commit_sha,
            token.as_deref(),
        )?;

        let working_dir = fs::canonicalize(&stack_path)
            .with_context(|| format!("Could not resolve stack folder {}", stack_path.display()))?;

        let config: Config = toml::from_str(
            &fs::read_to_string(working_dir.join("giong.toml")).with_context(|| {
                format!(
                    "Could not read {}",
                    working_dir.join("giong.toml").display()
                )
            })?,
        )
        .context("Invalid giong.toml")?;

        Ok((working_dir, config))
    }

    fn run_container(
        image: &str,
        working_dir: &Path,
        dockerfile: &str,
        logger: &mut File,
    ) -> Result<Option<i32>> {
        let directory = include_dir!("$CARGO_MANIFEST_DIR/dockerfiles")
            .get_dir(dockerfile)
            .with_context(|| format!("Dockerfile directory not found: {dockerfile}"))?;
        copy_dir(directory, working_dir).context("Could not copy toolchain files")?;
        let code = run_command(
            Command::new("docker")
                .args(["build", "--tag", image, "."])
                .current_dir(working_dir),
            logger,
        )?;
        if code != Some(0) {
            return Ok(code);
        }
        let result = run_command(Command::new("docker").args(["run", "--rm", image]), logger);
        let _ = run_command(Command::new("docker").args(["image", "rm", image]), logger);
        result
    }
}
