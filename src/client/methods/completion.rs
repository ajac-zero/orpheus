use crate::{
    client::{OrpheusCore, mode::Mode},
    models::completion::{CompletionRequest, CompletionRequestBuilder},
};

impl<'a, M: Mode> OrpheusCore<M> {
    /// Initialize a builder for a text completion request
    pub fn completion(&'a self, prompt: impl Into<String>) -> CompletionRequestBuilder<'a, M> {
        CompletionRequest::builder(&self.pool, prompt)
    }
}
