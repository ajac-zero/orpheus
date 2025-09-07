use crate::{
    client::{Orpheus, core::Sync},
    models::chat::{ChatRequest, ChatRequestBuilder, Messages},
};

impl Orpheus {
    /// Initialize a builder for a chat completion request
    pub fn chat(&self, messages: impl Into<Messages>) -> ChatRequestBuilder<Sync> {
        let handler = self.create_handler();
        ChatRequest::builder(
            #[cfg(feature = "otel")]
            crate::otel::chat_span(),
            Some(handler),
            messages,
        )
    }
}
