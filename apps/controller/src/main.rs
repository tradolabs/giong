use giong_core::{bootstrap, config::Config};
mod errors;

mod presentation;

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    let result = runtime.block_on(async {
        let application = bootstrap::ensure_initialized(Config::default()).await?;
        presentation::serve(application).await?;
        Ok(())
    });
    runtime.shutdown_background();
    result
}
