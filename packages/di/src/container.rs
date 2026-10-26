use crate::error::Error;
use crate::injectable::Injectable;
use crate::resolver::Resolver;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

pub(crate) type Instance = Arc<dyn Any + Send + Sync>;
type Factory = fn(&mut Resolver<'_>) -> Result<Instance, Error>;

pub(crate) struct Registration {
    pub(crate) instance: OnceLock<Instance>,
    pub(crate) factory: Option<Factory>,
}

#[derive(Default)]
pub struct Container {
    pub(crate) registrations: HashMap<TypeId, Registration>,
}

impl Container {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<T: Any + Send + Sync>(&mut self, instance: T) -> &mut Self {
        self.registrations.insert(
            TypeId::of::<T>(),
            Registration {
                instance: OnceLock::from(Arc::new(instance) as Instance),
                factory: None,
            },
        );
        self
    }

    pub fn register_type<T: Injectable>(&mut self) -> &mut Self {
        self.registrations.insert(
            TypeId::of::<T>(),
            Registration {
                instance: OnceLock::new(),
                factory: Some(|resolver| Ok(Arc::new(T::inject(resolver)?))),
            },
        );
        self
    }

    pub fn get<T: Any + Send + Sync>(&self) -> Result<Arc<T>, Error> {
        Resolver::new(self).get::<T>()
    }
}
