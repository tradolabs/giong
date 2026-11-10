use crate::application::exceptions::base::Exception;
use giong_database::Database;
use giong_database::entities::stack_entity::StackEntity;
use giong_database::enums::job_status::JobStatus;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use giong_runner::types::configuration_sources;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Injectable)]
pub struct StackService {
    database: Arc<Database>,
}

impl StackService {
    pub async fn save(
        &self,
        id: Option<Uuid>,
        name: String,
        configurations: Value,
    ) -> Result<StackEntity, Exception> {
        let name = name.trim().to_owned();
        if name.is_empty() || name.chars().any(char::is_control) {
            return Err(Exception::BadRequest(
                "Stack name is required and cannot contain control characters".into(),
            ));
        }
        if let Some(existing) =
            self.database.stacks.find_one(Condition::new().eq("name", name.as_str())).await?
        {
            if Some(existing.id) != id {
                return Err(Exception::Conflict("modules.stack.already_exists"));
            }
        }
        if let Some(id) = id {
            if !self.database.stacks.exists(Condition::new().eq("id", id)).await? {
                return Err(Exception::NotFound("modules.stack.not_found"));
            }
            if self
                .database
                .jobs
                .exists(
                    Condition::new()
                        .eq("stack_id", id)
                        .is_in("status", [JobStatus::Pending, JobStatus::Running]),
                )
                .await?
            {
                return Err(Exception::Conflict("modules.job.active_job_exists"));
            }
        }
        let sources = configuration_sources(&configurations)
            .map_err(|error| Exception::BadRequest(error.to_string()))?;
        for source in sources {
            self.database
                .repositories
                .find_one(Condition::new().eq("url", source.repo_url.as_str()))
                .await?
                .ok_or_else(|| {
                    Exception::BadRequest(format!(
                        "Register repository {} before using it in a source",
                        source.repo_url
                    ))
                })?;
        }
        let plan_name = name.clone();
        let plan_config = configurations.clone();
        tokio::task::spawn_blocking(move || giong_runner::plugins::plan(&plan_name, &plan_config))
            .await
            .map_err(Exception::unexpected)?
            .map_err(|error| Exception::BadRequest(error.to_string()))?;
        let stack = self
            .database
            .transaction(async || {
                let changes =
                    Document::new().set("name", name).set("configurations", configurations);
                let stack = if let Some(id) = id {
                    self.database.stacks.update_one(Condition::new().eq("id", id), changes).await?;
                    self.database.stacks.find_one(Condition::new().eq("id", id)).await?.ok_or_else(
                        || {
                            giong_database::error::Error::Unexpected(
                                "Stack no longer exists".into(),
                            )
                        },
                    )?
                } else {
                    self.database.stacks.create_one(changes).await?
                };
                Ok(stack)
            })
            .await?;
        Ok(stack)
    }
}
