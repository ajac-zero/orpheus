use super::Orpheus;
use crate::models::{ChatMessages, ChatRequest, ChatRequestBuilder, Sync};

impl Orpheus {
    /// Initialize a builder for a chat completion request
    pub fn chat(&self, messages: impl Into<ChatMessages>) -> ChatRequestBuilder<Sync> {
        let handler = self.create_handler();
        ChatRequest::builder(
            #[cfg(feature = "otel")]
            crate::models::otel::chat_span(),
            Some(handler),
            messages,
        )
    }
}
