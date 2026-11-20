use crate::application::exceptions::base::Exception;
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use giong_docker::images;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Injectable)]
pub struct GetNodeKindsUseCase {
    database: Arc<Database>,
}

impl UseCase for GetNodeKindsUseCase {
    type Input = ();
    type Output = Vec<String>;

    async fn execute(&self, (): Self::Input) -> Result<Self::Output, Exception> {
        let registered = self.database.plugins.find(Condition::new()).await?;
        let local_image_ids: HashSet<_> =
            images::get_images().await?.into_iter().map(|image| image.id).collect();
        Ok(registered
            .into_iter()
            .filter(|plugin| local_image_ids.contains(&plugin.image_id))
            .map(|plugin| plugin.kind)
            .collect())
    }
}
