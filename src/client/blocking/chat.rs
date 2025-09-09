use crate::{
    client::{Orpheus, core::Sync},
    models::chat::{ChatRequest, ChatRequestBuilder, History},
};

impl Orpheus {
    /// Initialize a builder for a chat completion request
    ///
    /// # Examples
    ///
    /// ```
    /// use orpheus::prelude::*;
    ///
    /// let client = Orpheus::new("your_api_key");
    ///
    /// let response = client
    ///     .chat("hello!")
    ///     .model("openai/gpt-4o")
    ///     .send()
    ///     .unwrap();
    /// ```
    pub fn chat(&self, messages: impl Into<History>) -> ChatRequestBuilder<Sync> {
        let handler = self.create_handler();
        ChatRequest::builder(
            #[cfg(feature = "otel")]
            crate::otel::chat_span(),
            Some(handler),
            messages,
        )
    }
}
