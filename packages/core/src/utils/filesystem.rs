use anyhow::{Context, Result};
use include_dir::{Dir, DirEntry};
use std::{fs, path::Path};

pub(crate) fn remove_dir(path: &Path) {
    if !path.exists() {
        return;
    }

    if let Err(error) = fs::remove_dir_all(path) {
        eprintln!("Could not remove {}: {error}", path.display());
    }
}

pub(crate) fn copy_dir(source: &Dir<'_>, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;

    for entry in source.entries() {
        let name = entry
            .path()
            .file_name()
            .context("Template entry has no filename")?;
        let target = destination.join(name);
        if target.is_symlink() {
            fs::remove_file(&target)?;
        }
        match entry {
            DirEntry::Dir(directory) => copy_dir(directory, &target)?,
            DirEntry::File(file) => fs::write(&target, file.contents())?,
        }
    }

    Ok(())
}
