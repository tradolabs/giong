pub mod events;
pub mod execution;
pub mod git;
pub mod nodes;
pub mod secrets;
pub mod storage;

use crate::tools::execution::Execution;
use crate::tools::nodes::Outputs;
use crate::tools::storage::Storage;
use std::path::PathBuf;

#[derive(Default)]
pub struct Tools {
    pub events: Option<crate::tools::events::Events>,
    pub secrets: Option<crate::tools::secrets::Secrets>,
    pub scope: Option<String>,
    pub workspace: Option<PathBuf>,
    pub storage: Option<Storage>,
    pub execution: Option<Execution>,
    pub outputs: Option<Outputs>,
}
