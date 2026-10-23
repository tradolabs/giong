use super::base_repository;
use crate::{infrastructure::models::job_model, repositories::job_repository::JobRepository};

pub(crate) type SeaOrmJobRepository = base_repository::SeaOrmRepository<job_model::Entity>;

impl JobRepository for SeaOrmJobRepository {}
