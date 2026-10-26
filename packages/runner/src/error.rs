use std::{io, path::PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not read {}: {source}", path.display())]
    ConfigRead {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("Invalid pipeline configuration: {0}")]
    ConfigParse(#[from] toml::de::Error),
    #[error("Dockerfile directory not found: {0}")]
    DockerfileNotFound(String),
    #[error("Could not copy toolchain files: {0}")]
    TemplateCopy(#[source] Box<Error>),
    #[error("Template entry has no filename")]
    TemplateEntryMissingFilename,
    #[error("Could not execute pipeline command: {0}")]
    Command(#[source] io::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
}
