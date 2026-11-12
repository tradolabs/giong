use crate::infrastructure::models::node;
use crate::infrastructure::repositories::base::SeaOrmRepository;
use crate::repositories::node::NodeRepository;
pub(crate) type SeaOrmNodeRepository = SeaOrmRepository<node::Entity>;
impl NodeRepository for SeaOrmNodeRepository {}
