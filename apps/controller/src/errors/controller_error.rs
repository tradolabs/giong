use giong_core::application::exceptions::base::Exception;

#[derive(Debug, thiserror::Error)]
pub(crate) enum ControllerError {
    #[error("JWT secret is not configured")]
    MissingJwtSecret,
    #[error(transparent)]
    Application(#[from] Exception),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Scheduler(#[from] giong_scheduler::Error),
}
