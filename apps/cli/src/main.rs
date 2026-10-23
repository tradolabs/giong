use anyhow::Result;
use clap::Parser;
use giong_core::Application;
use std::path::PathBuf;

mod command;
mod resource;

use command::Command;
use resource::Resource;

#[derive(Parser)]
#[command(name = "giong", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    let application = Application::new(cli.data_dir).await?;

    match cli.command {
        Command::Add(resource) => match resource {
            Resource::Repository { name, url, token } => {
                application.add_repo(&name, &url, token.as_deref()).await?;
                println!("Added repository {name}");
            }
            Resource::Stack { name, repo_url, branch, folder_path } => {
                application
                    .add_stack(&name, &repo_url, &branch, &folder_path)
                    .await?;
                println!("Added stack {name}");
            }
        },
    }

    Ok(())
}
