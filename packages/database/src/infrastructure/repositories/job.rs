use super::base::SeaOrmRepository;
use crate::{infrastructure::models::job, repositories::job::JobRepository};

pub(crate) type SeaOrmJobRepository = SeaOrmRepository<job::Entity>;

impl JobRepository for SeaOrmJobRepository {}
