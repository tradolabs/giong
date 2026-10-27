use crate::container::Container;
use crate::error::Error;
use std::any::{Any, TypeId, type_name};
use std::sync::Arc;

pub struct Resolver<'a> {
    container: &'a Container,
    stack: Vec<(TypeId, &'static str)>,
}

impl<'a> Resolver<'a> {
    pub(crate) fn new(container: &'a Container) -> Self {
        Self { container, stack: Vec::new() }
    }

    pub fn get<T: Any + Send + Sync>(&mut self) -> Result<Arc<T>, Error> {
        let id = TypeId::of::<T>();
        let name = type_name::<T>();
        let registration =
            self.container.registrations.get(&id).ok_or_else(|| Error::Missing(name.to_owned()))?;
        if let Some(instance) = registration.instance.get() {
            return Ok(Arc::clone(instance)
                .downcast::<T>()
                .expect("registration type matches its key"));
        }
        if let Some(start) = self.stack.iter().position(|(active, _)| *active == id) {
            let mut path: Vec<_> =
                self.stack[start..].iter().map(|(_, name)| (*name).to_owned()).collect();
            path.push(name.to_owned());
            return Err(Error::Cycle(path));
        }
        self.stack.push((id, name));
        let result = registration.factory.expect("uncached registration has a factory")(self);
        self.stack.pop();
        let instance = result?;
        let cached = registration.instance.get_or_init(|| instance);
        Ok(Arc::clone(cached).downcast::<T>().expect("registration type matches its key"))
    }

    pub fn get_optional<T: Any + Send + Sync>(&mut self) -> Result<Option<Arc<T>>, Error> {
        match self.get::<T>() {
            Ok(instance) => Ok(Some(instance)),
            Err(Error::Missing(name)) if name == type_name::<T>() => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn get_dyn<T: ?Sized + Send + Sync + 'static>(&mut self) -> Result<Arc<T>, Error> {
        self.get::<Arc<T>>().map(|instance| Arc::clone(instance.as_ref()))
    }

    pub fn get_optional_dyn<T: ?Sized + Send + Sync + 'static>(
        &mut self,
    ) -> Result<Option<Arc<T>>, Error> {
        Ok(self.get_optional::<Arc<T>>()?.map(|instance| Arc::clone(instance.as_ref())))
    }
}
