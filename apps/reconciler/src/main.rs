use giong_provider::runtime::Runtime;
use giong_storage::storage::Storage;
use std::path::PathBuf;
use std::{env, sync::Arc};
use tokio::net::TcpListener;

mod routes;
mod server;
mod snapshot;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let directory = PathBuf::from(env::var_os("HOME").expect("HOME must be set")).join(".giong");
    let runtime = Arc::new(Runtime::new(
        directory.join("providers"),
        "ghcr.io/tradolabs/giong-providers",
        "0.1.0",
    )?);
    let storage = Arc::new(Storage::new(&env::var("GIONG_STORAGE_URL")?)?);
    let listener = TcpListener::bind("0.0.0.0:2929").await?;
    eprintln!("Reconciler listening on {}", listener.local_addr()?);
    axum::serve(listener, server::router(runtime, storage)).await?;
    Ok(())
}
