use clap::Subcommand;

#[derive(Subcommand)]
pub(crate) enum Command {
    ResetAdminPassword,
    /// Run a YAML flow once.
    Run {
        file: std::path::PathBuf,
    },
    /// Store a secret in a vault, reading its value from stdin.
    SetSecret {
        #[arg(long)]
        vault_id: uuid::Uuid,
        #[arg(long)]
        key: String,
    },
    /// Run one stored node.
    RunNode {
        flow_id: uuid::Uuid,
        node_id: uuid::Uuid,
    },
    /// Delete a flow and its owned nodes and vault.
    DeleteFlow {
        flow_id: uuid::Uuid,
    },
    /// Process node schedules and change events continuously.
    Watch,
    #[command(subcommand)]
    Add(AddResource),
}

#[derive(Subcommand)]
pub(crate) enum AddResource {
    Vault {
        file: std::path::PathBuf,
    },
    Flow {
        #[arg(long, value_delimiter = ',')]
        inherited_vault_ids: Vec<uuid::Uuid>,
        #[arg(long)]
        name: String,
        /// Node definitions as a JSON array.
        #[arg(long, default_value = "[]")]
        nodes: String,
    },
}
