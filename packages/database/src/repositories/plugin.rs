use crate::entities::plugin_entity::PluginEntity;
use crate::repositories::base::Repository;

pub trait PluginRepository: Repository<PluginEntity> {}
