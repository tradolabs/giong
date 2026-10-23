use anyhow::Result;
use futures_util::future::{LocalBoxFuture, try_join_all};
use tokio::sync::mpsc;
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct Scheduler<'a> {
    scheduler: JobScheduler,
    tasks: Vec<LocalBoxFuture<'a, Result<()>>>,
}

impl<'a> Scheduler<'a> {
    pub async fn new() -> Result<Self> {
        Ok(Self { scheduler: JobScheduler::new().await?, tasks: Vec::new() })
    }

    pub async fn register(
        &mut self,
        cron: &str,
        mut callback: impl AsyncFnMut() -> Result<()> + 'a,
    ) -> Result<()> {
        let (sender, mut receiver) = mpsc::channel(1);
        sender.try_send(())?;

        self.scheduler
            .add(Job::new(cron, move |_, _| {
                let _ = sender.try_send(());
            })?)
            .await?;

        self.tasks.push(Box::pin(async move {
            while receiver.recv().await.is_some() {
                callback().await?;
            }
            Ok(())
        }));

        Ok(())
    }

    pub async fn start(self) -> Result<()> {
        self.scheduler.start().await?;
        try_join_all(self.tasks).await?;
        Ok(())
    }
}
