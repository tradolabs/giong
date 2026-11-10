use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Runner plugin: {0}")]
    Plugin(String),
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Unexpected(error.to_string())
    }
}
