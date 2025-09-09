use crate::{
    client::{Orpheus, core::Sync},
    models::completion::{CompletionRequest, CompletionRequestBuilder},
};

impl Orpheus {
    /// Initialize a builder for a text completion request
    ///
    /// # Examples
    ///
    /// ```
    /// use orpheus::prelude::*;
    ///
    /// let client = Orpheus::new("your_api_key");
    ///
    /// let response = client
    ///     .completion("The capital of France is ")
    ///     .model("openai/gpt-4o")
    ///     .send()
    ///     .unwrap();
    /// ```
    pub fn completion(&self, prompt: impl Into<String>) -> CompletionRequestBuilder<Sync> {
        let handler = self.create_handler();
        CompletionRequest::builder(Some(handler), prompt)
    }
}
