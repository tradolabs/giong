use clap::Subcommand;

#[derive(Subcommand)]
pub(crate) enum Resource {
    Repository {
        #[arg(long)]
        name: String,
        #[arg(long)]
        url: String,
        /// Personal access token
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
