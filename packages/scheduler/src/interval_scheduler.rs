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
    jobs: Mutex<HashMap<Uuid, ScheduledJob>>,
}

struct ScheduledJob {
    id: Uuid,
    seconds: u64,
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
        self.schedule_job(key, seconds, callback, false).await
    }

    /// Register a job unless one with the same key and interval is already scheduled.
    pub async fn ensure_scheduled<F, Fut>(
        &self,
        key: Uuid,
        seconds: u64,
        callback: F,
    ) -> Result<(), Error>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.schedule_job(key, seconds, callback, true).await
    }

    async fn schedule_job<F, Fut>(
        &self,
        key: Uuid,
        seconds: u64,
        callback: F,
        keep_existing: bool,
    ) -> Result<(), Error>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let mut jobs = self.jobs.lock().await;
        if let Some(current) = jobs.get(&key) {
            if keep_existing && current.seconds == seconds {
                return Ok(());
            }
            self.scheduler.remove(&current.id).await?;
            jobs.remove(&key);
        }
        let callback = Arc::new(callback);
        let job = Job::new_repeated_async(Duration::from_secs(seconds), move |_, _| {
            let callback = Arc::clone(&callback);
            Box::pin(async move {
                callback().await;
            })
        })?;
        let job_id = self.scheduler.add(job).await?;
        jobs.insert(key, ScheduledJob { id: job_id, seconds });
        Ok(())
    }

    pub async fn unschedule(&self, key: Uuid) -> Result<(), Error> {
        let mut jobs = self.jobs.lock().await;
        if let Some(job) = jobs.get(&key) {
            self.scheduler.remove(&job.id).await?;
            jobs.remove(&key);
        }
        Ok(())
    }
}
