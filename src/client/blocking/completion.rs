use crate::{
    client::{Orpheus, core::Sync},
    models::completion::{CompletionRequest, CompletionRequestBuilder},
};

impl Orpheus {
    /// Initialize a builder for a text completion request
    pub fn completion(&self, prompt: impl Into<String>) -> CompletionRequestBuilder<Sync> {
        let handler = self.create_handler();
        CompletionRequest::builder(Some(handler), prompt)
    }
}
