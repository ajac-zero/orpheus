use crate::{
    client::{OrpheusCore, mode::Mode},
    models::completion::{CompletionRequest, CompletionRequestBuilder},
};

impl<'a, M: Mode> OrpheusCore<M> {
    /// Initialize a builder for a text completion request
    pub fn completion(&'a self, prompt: impl Into<String>) -> CompletionRequestBuilder<'a, M> {
        let api_key = self.keystore.get_api_key().ok();
        CompletionRequest::builder(&self.pool, api_key, prompt)
    }
}
