use super::events::QueueEvent;
use crate::application::exceptions::base::Exception;

#[async_trait::async_trait]
pub trait QueueManager: Send + Sync {
    async fn enqueue(&self, event: QueueEvent) -> Result<(), Exception>;
}
