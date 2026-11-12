use giong_core::{bootstrap, config::Config};

mod presentation;

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    let result = runtime.block_on(async {
        let application = bootstrap::ensure_initialized(Config::default()).await?;
        use giong_core::application::use_cases::node_triggers::{
            DispatchNodeTriggersUseCase, ResumePendingRunsUseCase,
        };
        application.execute::<ResumePendingRunsUseCase>(()).await?;
        let triggers = application.clone();
        let scheduler = tokio::spawn(async move {
            loop {
                if let Err(error) = triggers.execute::<DispatchNodeTriggersUseCase>(()).await {
                    eprintln!("Node triggers: {error}");
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
        let result = presentation::serve(application).await;
        scheduler.abort();
        result?;
        Ok(())
    });
    runtime.shutdown_background();
    result
}
