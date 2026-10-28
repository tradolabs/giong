#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid resource type: {0}")]
    InvalidResourceType(String),

    #[error("Provider failed: {0}")]
    Provider(String),

    #[error("Unexpected error: {0}")]
    Unexpected(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    pub(crate) fn unexpected(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        let error: Box<dyn std::error::Error + Send + Sync> = error.into();
        error.into()
    }
}
