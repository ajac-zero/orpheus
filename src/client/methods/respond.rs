use crate::{
    client::{OrpheusCore, mode::Mode},
    models::{Input, request::ResponseRequestBuilder},
};

impl<'a, M: Mode> OrpheusCore<M> {
    /// Create a new response request.
    pub fn respond(&'a self, input: impl Into<Input>) -> ResponseRequestBuilder<'a, M> {
        ResponseRequestBuilder::new(self, input.into())
    }
}
