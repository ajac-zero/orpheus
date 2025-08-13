#[cfg(feature = "otel")]
use tracing::{Span, field::Empty, instrument};

use super::main::AsyncOrpheus;
use crate::models::chat::*;

impl AsyncOrpheus {
    #[cfg_attr(feature = "otel", instrument(
        name = "chat orpheus",
        fields(
            otel.kind = "client",
            otel.status_code = Empty,
            gen_ai.operation.name = "chat",
            gen_ai.system = "openrouter",
            gen_ai.output.type = Empty,
            gen_ai.request.choice.count = Empty,
            gen_ai.request.model = Empty,
            gen_ai.request.seed = Empty,
            gen_ai.request.frequency_penalty = Empty,
            gen_ai.request.max_tokens = Empty,
            gen_ai.request.presence_penalty = Empty,
            gen_ai.request.stop_sequences = Empty,
            gen_ai.request.temperature = Empty,
            gen_ai.request.top_k = Empty,
            gen_ai.request.top_p = Empty,
            gen_ai.response.finish_reasons = Empty,
            gen_ai.response.id = Empty,
            gen_ai.response.model = Empty,
            gen_ai.usage.input_tokens = Empty,
            gen_ai.usage.output_tokens = Empty,
        ),
        skip_all
    ))]
    pub fn chat(&self, messages: impl Into<ChatMessages>) -> ChatRequestBuilder<Async> {
        let handler = self.create_handler();
        ChatRequest::builder(
            #[cfg(feature = "otel")]
            Span::current(),
            Some(handler),
            messages,
        )
    }
}
