#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Event serialization failed: {0}")]
    SerializationError(String),
    #[error("Publisher backend failed: {0}")]
    Unexpected(#[source] Box<dyn std::error::Error + Send + Sync>),
}
