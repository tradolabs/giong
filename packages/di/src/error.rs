#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("dependency is not registered: {0}")]
    Missing(String),
    #[error("dependency cycle: {}", .0.join(" -> "))]
    Cycle(Vec<String>),
}
