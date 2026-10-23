use clap::Subcommand;

use crate::resource::Resource;

#[derive(Subcommand)]
pub(crate) enum Command {
    #[command(subcommand)]
    Add(Resource),
}
