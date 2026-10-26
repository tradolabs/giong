mod credentials;
mod error;
mod repository;

pub use error::Error;
pub use repository::{checkout, get_last_commit};
