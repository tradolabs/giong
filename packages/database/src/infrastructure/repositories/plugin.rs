use crate::infrastructure::models::plugin;
use crate::infrastructure::repositories::base::SeaOrmRepository;
use crate::repositories::plugin::PluginRepository;

pub(crate) type SeaOrmPluginRepository = SeaOrmRepository<plugin::Entity>;
impl PluginRepository for SeaOrmPluginRepository {}
