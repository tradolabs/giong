#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Flow,
    Node,
    Vault,
    Secret,
}

#[derive(Debug, thiserror::Error)]
pub enum ResourceException {
    #[error("{0:?} not found")]
    NotFound(Resource),
    #[error("{0:?} already exists")]
    AlreadyExists(Resource),
    #[error("{0:?} is in use")]
    InUse(Resource),
}
