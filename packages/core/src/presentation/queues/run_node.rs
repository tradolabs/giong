use crate::application::ports::queue::events::RunNodeEvent;
use crate::application::use_cases::run_node::{RunNodeInput, RunNodeUseCase};
use crate::application::use_cases::use_case::UseCase;
use giong_di::Container;
use std::sync::Arc;

pub(crate) async fn handle(container: Arc<Container>, event: RunNodeEvent) {
    tokio::spawn(async move {
        let run_id = event.run_id;
        let use_case = match container.get::<RunNodeUseCase>() {
            Ok(use_case) => use_case,
            Err(error) => {
                eprintln!("Node run {run_id}: {error:#}");
                return;
            }
        };
        if let Err(error) = use_case.execute(RunNodeInput { run_id }).await {
            eprintln!("Node run {run_id}: {error:#}");
        }
    });
}
