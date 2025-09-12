use crate::{
    client::core::{Mode, OrpheusCore},
    models::chat::{ChatRequest, ChatRequestBuilder, History},
};

impl<M: Mode> OrpheusCore<M> {
    /// Initialize a builder for a chat completion request
    pub fn chat(&self, messages: impl Into<History>) -> ChatRequestBuilder<M> {
        let handler = self.create_handler();
        ChatRequest::builder(
            #[cfg(feature = "otel")]
            crate::otel::chat_span(),
            Some(handler),
            messages,
        )
    }
}
