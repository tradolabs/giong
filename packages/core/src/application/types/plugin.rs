use giong_database::entities::plugin_entity::PluginEntity;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PluginStatus {
    Installed,
    Missing,
    Unknown,
}

#[derive(Serialize)]
pub struct PluginInfo {
    kind: String,
    image: String,
    name: String,
    version: String,
    description: String,
    status: PluginStatus,
}

#[derive(Serialize)]
pub struct PluginInventory {
    pub(crate) docker_available: bool,
    pub(crate) plugins: Vec<PluginInfo>,
}

impl PluginInfo {
    pub(crate) fn new(plugin: PluginEntity, status: PluginStatus) -> Self {
        Self {
            kind: plugin.kind,
            image: plugin.image,
            name: plugin.name,
            version: plugin.version,
            description: plugin.description,
            status,
        }
    }
}
