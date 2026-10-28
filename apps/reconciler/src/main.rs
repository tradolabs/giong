use giong_provider::runtime::Runtime;
use std::path::PathBuf;
use std::{env, sync::Arc};
use tokio::net::TcpListener;

mod server;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let directory = PathBuf::from(env::var_os("HOME").expect("HOME must be set"))
        .join(".giong")
        .join("providers");
    let runtime = Arc::new(Runtime::new(directory, "ghcr.io/tradolabs", "0.1.0")?);
    let listener = TcpListener::bind("0.0.0.0:2929").await?;
    eprintln!("Reconciler listening on {}", listener.local_addr()?);
    axum::serve(listener, server::router(runtime)).await?;
    Ok(())
}
