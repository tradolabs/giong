use clap::Subcommand;

#[derive(Subcommand)]
pub(crate) enum Command {
    ResetAdminPassword,
}
