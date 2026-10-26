#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Scheduler(#[from] tokio_cron_scheduler::JobSchedulerError),
    #[error(transparent)]
    Trigger(#[from] tokio::sync::mpsc::error::TrySendError<()>),
}
