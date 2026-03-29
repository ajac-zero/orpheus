use crate::{
    backend::Backend,
    client::OrpheusCore,
    models::{Input, request::ResponseRequestBuilder},
};

impl<'a, B: Backend> OrpheusCore<B> {
    /// Create a new response request.
    pub fn respond(&'a self, input: impl Into<Input>) -> ResponseRequestBuilder<'a, B> {
        ResponseRequestBuilder::new(self, input.into())
    }
}
