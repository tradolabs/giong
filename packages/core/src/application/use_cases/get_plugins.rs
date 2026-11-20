use crate::application::exceptions::base::Exception;
use crate::application::types::plugin::{PluginInfo, PluginInventory, PluginStatus};
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use giong_docker::images;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Injectable)]
pub struct GetPluginsUseCase {
    database: Arc<Database>,
}

impl UseCase for GetPluginsUseCase {
    type Input = ();
    type Output = PluginInventory;

    async fn execute(&self, (): Self::Input) -> Result<Self::Output, Exception> {
        let registered = self.database.plugins.find(Condition::new()).await?;
        let images = images::get_images().await;
        let docker_available = images.is_ok();
        let local_ids: HashSet<_> =
            images.iter().flatten().map(|image| image.id.as_str()).collect();
        let plugins = registered
            .into_iter()
            .map(|plugin| {
                let status = if !docker_available {
                    PluginStatus::Unknown
                } else if local_ids.contains(plugin.image_id.as_str()) {
                    PluginStatus::Installed
                } else {
                    PluginStatus::Missing
                };
                PluginInfo::new(plugin, status)
            })
            .collect();
        Ok(PluginInventory { docker_available, plugins })
    }
}
