#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("orm error: {0}")]
    Orm(String),
    #[error("invalid database configuration: {0}")]
    Configuration(String),
    #[error("unexpected error: {0}")]
    Unexpected(String),
}
