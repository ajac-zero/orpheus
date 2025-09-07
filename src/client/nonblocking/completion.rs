use crate::{
    client::{AsyncOrpheus, core::Async},
    models::completion::{CompletionRequest, CompletionRequestBuilder},
};

impl AsyncOrpheus {
    /// Send a text completion request
    pub fn completion(&self, prompt: impl Into<String>) -> CompletionRequestBuilder<Async> {
        let handler = self.create_handler();
        CompletionRequest::builder(Some(handler), prompt)
    }
}
