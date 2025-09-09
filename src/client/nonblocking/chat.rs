use crate::{
    client::{AsyncOrpheus, core::Async},
    models::chat::{ChatRequest, ChatRequestBuilder, History},
};

impl AsyncOrpheus {
    /// Initialize a builder for an async chat completion request
    ///
    /// # Examples
    ///
    /// ```
    /// use orpheus::prelude::*;
    ///
    /// let client = AsyncOrpheus::new("your_api_key");
    ///
    /// let response = client
    ///     .chat("hello!")
    ///     .model("openai/gpt-4o")
    ///     .send()
    ///     .await
    ///     .unwrap();
    /// ```
    pub fn chat(&self, messages: impl Into<History>) -> ChatRequestBuilder<Async> {
        let handler = self.create_handler();
        ChatRequest::builder(
            #[cfg(feature = "otel")]
            crate::otel::chat_span(),
            Some(handler),
            messages,
        )
    }
}
