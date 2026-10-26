use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

pub trait Event: Serialize + DeserializeOwned + Send + Sync + Clone + Debug + 'static {}

impl<T> Event for T where T: Serialize + DeserializeOwned + Send + Sync + Clone + Debug + 'static {}
