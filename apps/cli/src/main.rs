use clap::Parser;
use giong_core::config::Config;

mod command;

use crate::command::AddResource;
use command::Command;

#[derive(Parser)]
#[command(name = "giong", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    let application = giong_core::bootstrap::ensure_initialized(Config::default()).await?;
    match cli.command {
        Command::ResetAdminPassword => {
            let password = application.reset_admin_password().await?;
            println!("Admin password: {password}");
        }
        Command::Add(resource) => match resource {
            AddResource::Repository { name, url, token } => {
                application.add_repo(&name, &url, token.as_deref()).await?;
                println!("Added repository {name}");
            }
            AddResource::Stack { name, repo_url, branch, folder_path } => {
                application.add_stack(&name, &repo_url, &branch, &folder_path).await?;
                println!("Added stack {name}");
            }
        },
    }

    Ok(())
}
