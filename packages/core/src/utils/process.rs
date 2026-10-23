use anyhow::{Context, Result};
use std::{
    fs::File,
    io::Write,
    process::{Command, Stdio},
};

pub(crate) fn run_command(command: &mut Command, log_path: &mut File) -> Result<Option<i32>> {
    writeln!(log_path, "\n$ {command:?}")?;
    log_path.flush()?;
    let status = command
        .stdin(Stdio::null())
        .stdout(log_path.try_clone()?)
        .stderr(log_path.try_clone()?)
        .status()
        .context("Could not execute pipeline command")?;
    Ok(status.code())
}
