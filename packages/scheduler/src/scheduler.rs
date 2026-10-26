use crate::error::Error;
use futures_util::future::{LocalBoxFuture, join_all};
use tokio::sync::mpsc;
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct Scheduler<'a> {
    scheduler: JobScheduler,
    tasks: Vec<LocalBoxFuture<'a, ()>>,
}

impl<'a> Scheduler<'a> {
    pub async fn new() -> Result<Self, Error> {
        Ok(Self { scheduler: JobScheduler::new().await?, tasks: Vec::new() })
    }

    pub async fn register(
        &mut self,
        cron: &str,
        mut callback: impl AsyncFnMut() + 'a,
    ) -> Result<(), Error> {
        let (sender, mut receiver) = mpsc::channel(1);
        sender.try_send(())?;

        self.scheduler
            .add(Job::new(cron, move |_, _| {
                let _ = sender.try_send(());
            })?)
            .await?;

        self.tasks.push(Box::pin(async move {
            while receiver.recv().await.is_some() {
                callback().await;
            }
        }));

        Ok(())
    }

    pub async fn start(self) -> Result<(), Error> {
        self.scheduler.start().await?;
        join_all(self.tasks).await;
        Ok(())
    }
}
