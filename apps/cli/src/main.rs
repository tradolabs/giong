use clap::Parser;
use giong_core::config::Config;

mod actions;
mod command;

use crate::command::Command;

#[derive(Parser)]
#[command(name = "giong", version, disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    let application = giong_core::bootstrap::ensure_cli_initialized(Config::default()).await?;
    match cli.command {
        Command::ResetAdminPassword => actions::reset_admin_password::execute(&application).await,
    }
}
