use clap::Subcommand;

#[derive(Subcommand)]
pub(crate) enum Command {
    ResetAdminPassword,
    #[command(subcommand)]
    Add(AddResource),
}

#[derive(Subcommand)]
pub(crate) enum AddResource {
    Repository {
        #[arg(long)]
        name: String,
        #[arg(long)]
        url: String,
        #[arg(long)]
        token: Option<String>,
    },
    Stack {
        #[arg(long)]
        name: String,
        #[arg(long)]
        repo_url: String,
        #[arg(long)]
        branch: String,
        #[arg(long, default_value = ".")]
        folder_path: String,
    },
}
