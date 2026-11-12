mod credentials;
mod error;
mod repository;

pub use error::Error;
pub use repository::{changed_files, checkout, get_last_commit, head, resolve_revision};
