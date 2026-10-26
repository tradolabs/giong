#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Git(#[from] git2::Error),
    #[error("{0}")]
    Unexpected(&'static str),
}
