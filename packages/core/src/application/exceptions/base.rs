use super::authentication::AuthenticationException;
use crate::application::exceptions::resource::ResourceException;

#[derive(Debug, thiserror::Error)]
pub enum Exception {
    #[error("{0}")]
    BadRequest(String),
    #[error(transparent)]
    Resource(#[from] ResourceException),
    #[error(transparent)]
    Authentication(#[from] AuthenticationException),
    #[error("{0}")]
    Unexpected(String),
}

impl Exception {
    pub fn unexpected(message: impl ToString) -> Self {
        Self::Unexpected(message.to_string())
    }
}

impl From<anyhow::Error> for Exception {
    fn from(error: anyhow::Error) -> Self {
        Self::Unexpected(error.to_string())
    }
}

impl From<giong_di::Error> for Exception {
    fn from(error: giong_di::Error) -> Self {
        Self::Unexpected(error.to_string())
    }
}

impl From<giong_database::error::Error> for Exception {
    fn from(error: giong_database::error::Error) -> Self {
        Self::Unexpected(error.to_string())
    }
}

impl From<giong_docker::error::Error> for Exception {
    fn from(error: giong_docker::error::Error) -> Self {
        Self::unexpected(error)
    }
}

impl From<serde_json::Error> for Exception {
    fn from(error: serde_json::Error) -> Self {
        Self::Unexpected(error.to_string())
    }
}

impl From<giong_kv::error::Error> for Exception {
    fn from(error: giong_kv::error::Error) -> Self {
        Self::Unexpected(error.to_string())
    }
}

impl From<std::io::Error> for Exception {
    fn from(error: std::io::Error) -> Self {
        Self::Unexpected(error.to_string())
    }
}

impl From<giong_scheduler::Error> for Exception {
    fn from(error: giong_scheduler::Error) -> Self {
        Self::Unexpected(error.to_string())
    }
}
