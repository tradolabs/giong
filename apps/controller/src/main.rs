use crate::scheduler::Scheduler;
use anyhow::Result;
use giong_core::Application;

mod http;
mod scheduler;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let application = Application::new(None).await?;

    tokio::try_join!(
        application.consume_jobs(),
        schedule(&application),
        http::serve(&application)
    )?;

    Ok(())
}

pub async fn schedule(application: &Application) -> Result<()> {
    if std::env::var("GIONG_SCHEDULE_ENABLED").as_deref() != Ok("true") {
        return Ok(());
    }

    let mut scheduler = Scheduler::new().await?;
    scheduler
        .register("*/30 * * * * *", async || {
            application.poll_repositories(None).await
        })
        .await?;

    scheduler.start().await
}
