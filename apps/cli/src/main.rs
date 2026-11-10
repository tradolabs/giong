use clap::Parser;
use giong_core::application::use_cases::create_repository::CreateRepositoryInput;
use giong_core::application::use_cases::create_stack::CreateStackInput;
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
                application
                    .create_repository(CreateRepositoryInput { name: name.clone(), url, token })
                    .await?;
                println!("Added repository {name}");
            }
            AddResource::Stack { name, repo_url, branch, folder_path } => {
                application
                    .create_stack(CreateStackInput {
                        name: name.clone(),
                        repo_url,
                        branch,
                        folder_path,
                    })
                    .await?;
                println!("Added stack {name}");
            }
        },
    }

    Ok(())
}
