use crate::application::ports::queue::events::RunJobEvent;
use crate::application::use_cases::run_job::{RunJobInput, RunJobUseCase};
use crate::application::use_cases::use_case::UseCase;
use giong_di::Container;
use std::sync::Arc;

pub(crate) async fn handle(container: Arc<Container>, event: RunJobEvent) {
    tokio::spawn(async move {
        let job_id = event.job_id;
        let use_case = match container.get::<RunJobUseCase>() {
            Ok(use_case) => use_case,
            Err(error) => {
                eprintln!("Job {job_id}: {error:#}");
                return;
            }
        };
        if let Err(error) = use_case.execute(RunJobInput { job_id }).await {
            eprintln!("Job {job_id}: {error:#}");
        }
    });
}
