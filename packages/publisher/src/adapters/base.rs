use crate::error::Error;
use crate::event::Event;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type Handler<E> =
    Arc<dyn Fn(E) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> + Send + Sync>;

#[async_trait::async_trait]
pub trait Adapter: Send + Sync {
    async fn publish<E: Event>(&self, event: E) -> Result<(), Error>;

    async fn subscribe<E: Event>(&self, handler: Handler<E>) -> Result<(), Error>;

    async fn shutdown(&self) -> Result<(), Error>;
}
