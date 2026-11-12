use crate::application::helpers::filesystem::root_dir;
use anyhow::Result;
use chrono::Utc;
use std::{
    fs::{self, File, OpenOptions},
    path::PathBuf,
};
use uuid::Uuid;

pub fn timestamp() -> String {
    Utc::now().format("%Y%m%dT%H%M%S%.9fZ").to_string()
}
pub fn path(node_id: Uuid, timestamp: &str) -> PathBuf {
    root_dir().join("logs").join("nodes").join(node_id.to_string()).join(format!("{timestamp}.log"))
}
pub fn open(node_id: Uuid, timestamp: &str) -> Result<File> {
    let path = path(node_id, timestamp);
    let directory = path.parent().expect("Node log has a parent directory");
    fs::create_dir_all(directory)?;
    let file = OpenOptions::new().create(true).append(true).open(&path)?;
    let mut previous = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && entry.path() != path
            && entry.path().extension().is_some_and(|extension| extension == "log")
        {
            previous.push(entry.path());
        }
    }
    previous.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    // Keep the active log and the nine most recent previous logs.
    for old in previous.into_iter().skip(9) {
        fs::remove_file(old)?;
    }
    Ok(file)
}
