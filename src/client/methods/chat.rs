use crate::{
    client::{OrpheusCore, mode::Mode},
    models::chat::{ChatRequest, ChatRequestBuilder, History},
};

impl<'a, M: Mode> OrpheusCore<M> {
    /// Initialize a builder for a chat completion request
    pub fn chat(&'a self, messages: impl Into<History>) -> ChatRequestBuilder<'a, M> {
        ChatRequest::builder(
            #[cfg(feature = "otel")]
            crate::otel::chat_span(),
            &self.pool,
            self.keystore.get_api_key().ok(),
            messages,
        )
    }
}
