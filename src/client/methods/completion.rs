use crate::{
    client::{Mode, OrpheusCore},
    models::completion::{CompletionRequest, CompletionRequestBuilder},
};

impl<M: Mode> OrpheusCore<M> {
    /// Initialize a builder for a text completion request
    pub fn completion(&self, prompt: impl Into<String>) -> CompletionRequestBuilder<M> {
        let handler = self.create_handler();
        CompletionRequest::builder(Some(handler), prompt)
    }
}
