use super::AsyncOrpheus;
use crate::models::{chat::*, common::mode::Async};

impl AsyncOrpheus {
    pub fn chat(&self, messages: impl Into<ChatMessages>) -> ChatRequestBuilder<Async> {
        let handler = self.create_handler();
        ChatRequest::builder(
            #[cfg(feature = "otel")]
            otel::chat_span(),
            Some(handler),
            messages,
        )
    }
}
