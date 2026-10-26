use crate::error::Error;
use crate::resolver::Resolver;
use std::any::Any;

pub trait Injectable: Any + Send + Sync + Sized {
    fn inject(resolver: &mut Resolver<'_>) -> Result<Self, Error>;
}
