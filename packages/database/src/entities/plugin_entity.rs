#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginEntity {
    pub kind: String,
    pub image: String,
    pub image_id: String,
    pub name: String,
    pub version: String,
    pub description: String,
}
