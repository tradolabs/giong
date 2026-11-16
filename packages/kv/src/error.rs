#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No entry exists for the given key")]
    NotFound,
    #[error("Unexpected: {0}")]
    Unexpected(#[source] Box<redb::Error>),
}

impl<E: Into<redb::Error>> From<E> for Error {
    fn from(error: E) -> Self {
        Self::Unexpected(Box::new(error.into()))
    }
}
