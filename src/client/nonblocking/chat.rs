use crate::{
    client::{AsyncOrpheus, core::Async},
    models::chat::{ChatRequest, ChatRequestBuilder, Messages},
};

impl AsyncOrpheus {
    pub fn chat(&self, messages: impl Into<Messages>) -> ChatRequestBuilder<Async> {
        let handler = self.create_handler();
        ChatRequest::builder(
            #[cfg(feature = "otel")]
            crate::otel::chat_span(),
            Some(handler),
            messages,
        )
    }
}
