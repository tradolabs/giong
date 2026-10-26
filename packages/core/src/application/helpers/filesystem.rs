use std::path::PathBuf;

pub(crate) fn root_dir() -> PathBuf {
    PathBuf::from(std::env::var_os("HOME").expect("HOME must be set")).join(".giong")
}

pub(crate) fn remove_dir(path: &std::path::Path) {
    if !path.exists() {
        return;
    }
    if let Err(error) = std::fs::remove_dir_all(path) {
        eprintln!("Could not remove {}: {error}", path.display());
    }
}
