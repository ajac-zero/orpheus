use crate::{
    client::{AsyncOrpheus, core::Async},
    models::chat::{ChatRequest, ChatRequestBuilder, History},
};

impl AsyncOrpheus {
    /// Initialize a builder for an async chat completion request
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
