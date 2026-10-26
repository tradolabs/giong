use crate::error::Error;
use std::{
    fs::File,
    io::Write,
    process::{Command, Stdio},
};

pub(crate) fn run_command(
    command: &mut Command,
    log_path: &mut File,
) -> Result<Option<i32>, Error> {
    writeln!(log_path, "\n$ {command:?}")?;
    log_path.flush()?;
    let status = command
        .stdin(Stdio::null())
        .stdout(log_path.try_clone()?)
        .stderr(log_path.try_clone()?)
        .status()
        .map_err(Error::Command)?;
    Ok(status.code())
}
