use crate::{
    client::{AsyncOrpheus, Orpheus},
    models::{
        Input,
        request::{AsyncResponseRequestBuilder, ResponseRequestBuilder},
    },
};

impl<'a> Orpheus {
    /// Create a new response request.
    pub fn respond(&'a self, input: impl Into<Input>) -> ResponseRequestBuilder<'a> {
        ResponseRequestBuilder::new(self, input.into())
    }
}

impl<'a> AsyncOrpheus {
    /// Create a new response request.
    pub fn respond(&'a self, input: impl Into<Input>) -> AsyncResponseRequestBuilder<'a> {
        AsyncResponseRequestBuilder::new(self, input.into())
    }
}
