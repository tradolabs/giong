use crate::error::Error;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;

pub struct IntervalScheduler {
    scheduler: JobScheduler,
    jobs: Mutex<HashMap<Uuid, Uuid>>,
}

impl IntervalScheduler {
    pub async fn new() -> Result<Self, Error> {
        let scheduler = JobScheduler::new().await?;
        scheduler.start().await?;
        Ok(Self { scheduler, jobs: Mutex::new(HashMap::new()) })
    }

    pub async fn schedule<F, Fut>(&self, key: Uuid, seconds: u64, callback: F) -> Result<(), Error>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.unschedule(key).await?;
        let callback = Arc::new(callback);
        let job = Job::new_repeated_async(Duration::from_secs(seconds), move |_, _| {
            let callback = Arc::clone(&callback);
            Box::pin(async move {
                callback().await;
            })
        })?;
        let job_id = self.scheduler.add(job).await?;
        self.jobs.lock().await.insert(key, job_id);
        Ok(())
    }

    pub async fn unschedule(&self, key: Uuid) -> Result<(), Error> {
        if let Some(job_id) = self.jobs.lock().await.remove(&key) {
            self.scheduler.remove(&job_id).await?;
        }
        Ok(())
    }
}
