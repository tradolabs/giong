use crate::application::exceptions::base::Exception;
use std::future::Future;

pub trait UseCase {
    type Input;
    type Output;

    fn execute(
        &self,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, Exception>> + Send;
}
