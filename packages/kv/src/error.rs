#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Key-value storage: {0}")]
    Backend(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl From<redb::DatabaseError> for Error {
    fn from(error: redb::DatabaseError) -> Self {
        Self::Backend(Box::new(error))
    }
}

impl From<redb::TransactionError> for Error {
    fn from(error: redb::TransactionError) -> Self {
        Self::Backend(Box::new(error))
    }
}

impl From<redb::TableError> for Error {
    fn from(error: redb::TableError) -> Self {
        Self::Backend(Box::new(error))
    }
}

impl From<redb::StorageError> for Error {
    fn from(error: redb::StorageError) -> Self {
        Self::Backend(Box::new(error))
    }
}

impl From<redb::CommitError> for Error {
    fn from(error: redb::CommitError) -> Self {
        Self::Backend(Box::new(error))
    }
}
