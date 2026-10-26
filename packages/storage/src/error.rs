#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("invalid storage configuration: {0}")]
    Configuration(String),
    #[error("invalid storage request: {0}")]
    BadRequest(String),
}
