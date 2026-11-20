use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Unexpected(#[from] io::Error),

    #[error("Docker returned an invalid image ID")]
    InvalidImageId,
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        let kind = if error.is_timeout() { io::ErrorKind::TimedOut } else { io::ErrorKind::Other };
        io::Error::new(kind, error).into()
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, error).into()
    }
}
