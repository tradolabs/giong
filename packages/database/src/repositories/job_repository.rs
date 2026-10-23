use super::base_repository::BaseRepository;
use crate::entities::job_entity::JobEntity;

pub trait JobRepository: BaseRepository<JobEntity> {}
