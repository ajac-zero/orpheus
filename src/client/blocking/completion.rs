use super::Orpheus;
use crate::models::{
    common::mode::Sync,
    completion::{CompletionRequest, CompletionRequestBuilder},
};

impl Orpheus {
    /// Send a text completion request
    pub fn completion(&self, prompt: impl Into<String>) -> CompletionRequestBuilder<Sync> {
        let handler = self.create_handler();
        CompletionRequest::builder(Some(handler), prompt)
    }
}
