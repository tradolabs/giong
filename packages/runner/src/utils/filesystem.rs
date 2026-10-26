use crate::error::Error;
use include_dir::{Dir, DirEntry};
use std::{fs, path::Path};

pub(crate) fn copy_dir(source: &Dir<'_>, destination: &Path) -> Result<(), Error> {
    fs::create_dir_all(destination)?;

    for entry in source.entries() {
        let name = entry.path().file_name().ok_or(Error::TemplateEntryMissingFilename)?;
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
