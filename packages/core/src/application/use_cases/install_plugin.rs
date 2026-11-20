use crate::application::exceptions::base::Exception;
use crate::application::types::plugin::{PluginInfo, PluginStatus};
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::entities::plugin_entity::PluginEntity;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use giong_docker::images;
use giong_node::runtime::{Plugin, validate_plugin_kind};
use std::sync::Arc;

pub struct InstallPluginInput {
    pub image: String,
    pub pull: bool,
}

#[derive(Injectable)]
pub struct InstallPluginUseCase {
    database: Arc<Database>,
}

impl UseCase for InstallPluginUseCase {
    type Input = InstallPluginInput;
    type Output = PluginInfo;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        let image = input.image.trim();

        if input.pull {
            images::pull_image(image)
                .await
                .map_err(|error| Exception::BadRequest(error.to_string()))?;
        }

        let image_id = images::inspect_image(image)
            .await
            .map_err(|error| Exception::BadRequest(error.to_string()))?;
        let metadata = Plugin::from_image(image_id.clone()).metadata().await.map_err(|error| {
            Exception::BadRequest(format!("Could not read plugin metadata: {error:#}"))
        })?;
        validate_plugin_kind(&metadata.kind)
            .map_err(|error| Exception::BadRequest(error.to_string()))?;

        let plugin = PluginEntity {
            kind: metadata.kind,
            image: image.into(),
            image_id,
            name: metadata.name,
            version: metadata.version,
            description: metadata.description,
        };
        let document = Document::new()
            .set("image", plugin.image.clone())
            .set("image_id", plugin.image_id.clone())
            .set("name", plugin.name.clone())
            .set("version", plugin.version.clone())
            .set("description", plugin.description.clone());
        self.database
            .transaction(async || {
                let plugins = &self.database.plugins;
                let condition = Condition::new().eq("kind", plugin.kind.clone());
                if !plugins.update_one(condition, document.clone()).await? {
                    plugins.create_one(document.set("kind", plugin.kind.clone())).await?;
                }
                Ok(())
            })
            .await?;
        Ok(PluginInfo::new(plugin, PluginStatus::Installed))
    }
}
