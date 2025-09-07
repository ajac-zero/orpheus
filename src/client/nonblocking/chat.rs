use super::AsyncOrpheus;
use crate::models::{Async, ChatMessages, ChatRequest, ChatRequestBuilder};

impl AsyncOrpheus {
    pub fn chat(&self, messages: impl Into<ChatMessages>) -> ChatRequestBuilder<Async> {
        let handler = self.create_handler();
        ChatRequest::builder(
            #[cfg(feature = "otel")]
            crate::models::otel::chat_span(),
            Some(handler),
            messages,
        )
    }
}
