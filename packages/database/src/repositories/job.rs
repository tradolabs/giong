use super::base::Repository;
use crate::entities::job_entity::JobEntity;

pub trait JobRepository: Repository<JobEntity> {}
