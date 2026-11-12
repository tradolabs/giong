use std::path::PathBuf;

pub(crate) fn root_dir() -> PathBuf {
    std::env::var_os("GIONG_DATA_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").expect("HOME must be set")).join(".giong")
        })
}

pub(crate) fn remove_dir(path: &std::path::Path) {
    if !path.exists() {
        return;
    }
    if let Err(error) = std::fs::remove_dir_all(path) {
        eprintln!("Could not remove {}: {error}", path.display());
    }
}
