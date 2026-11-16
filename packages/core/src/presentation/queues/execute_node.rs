use crate::application::ports::queue::events::ExecuteNodeEvent;
use crate::application::use_cases::execute_node::{ExecuteNodeInput, ExecuteNodeUseCase};
use crate::application::use_cases::use_case::UseCase;
use giong_di::Container;
use std::sync::Arc;

pub(crate) async fn handle(container: Arc<Container>, event: ExecuteNodeEvent) {
    tokio::spawn(async move {
        let node_id = event.node_id;
        let use_case = match container.get::<ExecuteNodeUseCase>() {
            Ok(use_case) => use_case,
            Err(error) => {
                eprintln!("Node {node_id}: {error:#}");
                return;
            }
        };
        if let Err(error) = use_case.execute(ExecuteNodeInput { node_id }).await {
            eprintln!("Node {node_id}: {error:#}");
        }
    });
}
