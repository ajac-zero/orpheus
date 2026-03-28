use crate::{
    client::OrpheusCore,
    models::{Input, request::ResponseRequestBuilder},
};

impl<'a, M: open_responses::client::Mode> OrpheusCore<M> {
    /// Create a new response request.
    pub fn respond(&'a self, input: impl Into<Input>) -> ResponseRequestBuilder<'a, M> {
        ResponseRequestBuilder::new(self, input.into())
    }
}
