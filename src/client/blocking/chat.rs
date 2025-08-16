use super::Orpheus;
use crate::models::{chat::*, common::mode::Sync};

impl Orpheus {
    /// Initialize a builder for a chat completion request
    pub fn chat(&self, messages: impl Into<ChatMessages>) -> ChatRequestBuilder<Sync> {
        let handler = self.create_handler();
        ChatRequest::builder(
            #[cfg(feature = "otel")]
            otel::chat_span(),
            Some(handler),
            messages,
        )
    }
}
