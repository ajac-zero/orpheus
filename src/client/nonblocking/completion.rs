use crate::{
    client::{AsyncOrpheus, core::Async},
    models::completion::{CompletionRequest, CompletionRequestBuilder},
};

impl AsyncOrpheus {
    /// Initialize a builder for an async text completion request
    ///
    /// # Examples
    ///
    /// ```
    /// use orpheus::prelude::*;
    ///
    /// let client = AsyncOrpheus::new("your_api_key");
    ///
    /// let response = client
    ///     .completion("The capital of France is ")
    ///     .model("openai/gpt-4o")
    ///     .send()
    ///     .await
    ///     .unwrap();
    /// ```
    pub fn completion(&self, prompt: impl Into<String>) -> CompletionRequestBuilder<Async> {
        let handler = self.create_handler();
        CompletionRequest::builder(Some(handler), prompt)
    }
}
