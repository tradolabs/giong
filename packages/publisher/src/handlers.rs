use crate::adapters::base::{Adapter, Handler};
use crate::adapters::default::DefaultAdapter;
use crate::error::Error;
use crate::event::Event;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Weak};

type Registration = Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>;

pub struct PublisherHandlers<S, A = DefaultAdapter> {
    adapter: Arc<A>,
    state: Weak<S>,
    registrations: Vec<Registration>,
}

impl<S: Send + Sync + 'static, A: Adapter + 'static> PublisherHandlers<S, A> {
    pub(crate) fn new(adapter: Arc<A>, state: Arc<S>) -> Self {
        Self { adapter, state: Arc::downgrade(&state), registrations: Vec::new() }
    }

    pub fn handler<E, H, Fut>(mut self, handler: H) -> Self
    where
        E: Event,
        H: Fn(Arc<S>, E) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let adapter = self.adapter.clone();
        let state = self.state.clone();
        let handler = Arc::new(handler);
        self.registrations.push(Box::pin(async move {
            let callback: Handler<E> = Arc::new(move |event| {
                let state = state.clone();
                let handler = handler.clone();
                Box::pin(async move {
                    if let Some(state) = state.upgrade() {
                        handler(state, event).await;
                    }
                    Ok(())
                })
            });
            adapter.subscribe(callback).await
        }));
        self
    }

    /// Register all handlers. The caller must retain an `Arc` to the state;
    /// subscriptions hold weak references to avoid ownership cycles.
    pub async fn register(self) -> Result<(), Error> {
        for registration in self.registrations {
            registration.await?;
        }
        Ok(())
    }
}
