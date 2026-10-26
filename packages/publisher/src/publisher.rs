use crate::adapters::base::Adapter;
use crate::adapters::default::DefaultAdapter;
use crate::error::Error;
use crate::event::Event;
use crate::handlers::PublisherHandlers;
use std::sync::Arc;

pub struct Publisher<A = DefaultAdapter> {
    adapter: Arc<A>,
}

impl Publisher<DefaultAdapter> {
    pub async fn new() -> Result<Self, Error> {
        let adapter = Arc::new(DefaultAdapter::new().await?);
        Ok(Self { adapter })
    }
}

impl<A: Adapter + 'static> Publisher<A> {
    pub fn with_adapter(adapter: Arc<A>) -> Self {
        Self { adapter }
    }

    pub async fn publish<E: Event>(&self, event: E) -> Result<(), Error> {
        self.adapter.publish(event).await
    }

    pub fn with_state<S: Send + Sync + 'static>(&self, state: Arc<S>) -> PublisherHandlers<S, A> {
        PublisherHandlers::new(self.adapter.clone(), state)
    }

    pub async fn shutdown(&self) -> Result<(), Error> {
        self.adapter.shutdown().await
    }
}
