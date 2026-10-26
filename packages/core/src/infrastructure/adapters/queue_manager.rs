use crate::application::exceptions::base::Exception;
use crate::application::ports::queue::{events::QueueEvent, queue_manager::QueueManager};
use giong_publisher::{Error, Publisher};
use std::sync::Arc;

pub(crate) struct QueueManagerImpl {
    publisher: Arc<Publisher>,
}

impl QueueManagerImpl {
    pub(crate) fn new(publisher: Arc<Publisher>) -> Self {
        Self { publisher }
    }
}

#[async_trait::async_trait]
impl QueueManager for QueueManagerImpl {
    async fn enqueue(&self, event: QueueEvent) -> Result<(), Exception> {
        event.publish(&self.publisher).await.map_err(Exception::unexpected)
    }
}

impl QueueEvent {
    pub(super) async fn publish(self, publisher: &Publisher) -> Result<(), Error> {
        match self {
            Self::RunJob(event) => publisher.publish(event).await,
        }
    }
}
