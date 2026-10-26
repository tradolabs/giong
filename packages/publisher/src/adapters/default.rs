use crate::adapters::base::{Adapter, Handler};
use crate::error::Error;
use crate::event::Event;
use tokio_events::EventBus;

#[derive(Clone, Debug)]
struct TokioMessage<E>(E);

impl<E: Event> tokio_events::Event for TokioMessage<E> {
    fn event_type() -> &'static str {
        std::any::type_name::<E>()
    }

    fn serialize_event(&self) -> tokio_events::Result<Vec<u8>> {
        serde_json::to_vec(&self.0)
            .map_err(|error| tokio_events::Error::SerializationError(error.to_string()))
    }

    fn deserialize_event(bytes: &[u8]) -> tokio_events::Result<Self> {
        serde_json::from_slice(bytes)
            .map(Self)
            .map_err(|error| tokio_events::Error::SerializationError(error.to_string()))
    }
}

pub struct DefaultAdapter {
    event_bus: EventBus,
}

impl DefaultAdapter {
    pub async fn new() -> Result<Self, Error> {
        let event_bus = EventBus::builder()
            .build()
            .await
            .map_err(|error| Error::Unexpected(Box::new(error)))?;
        Ok(Self { event_bus })
    }
}

#[async_trait::async_trait]
impl Adapter for DefaultAdapter {
    async fn publish<E: Event>(&self, event: E) -> Result<(), Error> {
        self.event_bus
            .publish(TokioMessage(event))
            .await
            .map(|_| ())
            .map_err(|error| Error::Unexpected(Box::new(error)))
    }

    async fn subscribe<E: Event>(&self, handler: Handler<E>) -> Result<(), Error> {
        self.event_bus
            .subscribe_fallible(move |message: TokioMessage<E>| {
                let handler = handler.clone();
                async move {
                    handler(message.0)
                        .await
                        .map_err(|error| tokio_events::Error::SerializationError(error.to_string()))
                }
            })
            .await
            .map_err(|error| Error::Unexpected(Box::new(error)))?
            .detach();
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), Error> {
        self.event_bus.shutdown().await.map_err(|error| Error::Unexpected(Box::new(error)))
    }
}
