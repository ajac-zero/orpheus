use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatStreamChunk {
    /// Unique identifier for the chat completion
    pub id: String,

    /// The provider of the model
    pub provider: Option<String>,

    /// The model used for the completion
    pub model: Option<String>,

    /// The object type (always "chat.completion.chunk" for streaming)
    pub object: String,

    /// Unix timestamp of when the completion was created
    pub created: i64,

    /// List of streaming choices
    pub choices: Vec<ChatStreamChoice>,

    /// System fingerprint for the response
    pub system_fingerprint: Option<String>,

    /// Usage statistics (only present in the final chunk)
    pub usage: Option<super::ChatUsage>,
}

impl ChatStreamChunk {
    pub fn delta(&self) -> Result<&super::Message> {
        let message = &self
            .choices
            .iter()
            .next()
            .ok_or(Error::malformed_response(
                "Choices array in response is empty",
            ))?
            .delta;

        Ok(message)
    }

    pub fn into_delta(self) -> Result<super::Message> {
        let message = self
            .choices
            .into_iter()
            .next()
            .ok_or(Error::malformed_response(
                "Choices array in response is empty",
            ))?
            .delta;

        Ok(message)
    }

    pub fn into_content(self) -> Result<super::Content> {
        Ok(self.into_delta()?.content)
    }

    pub fn content(&self) -> Result<&super::Content> {
        Ok(&self.delta()?.content)
    }

    pub fn reasoning(&self) -> Result<Option<&String>> {
        Ok(self.delta()?.reasoning.as_ref())
    }
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatStreamChoice {
    /// The index of the choice
    pub index: u8,

    /// The delta containing incremental message content
    pub delta: super::Message,

    /// The reason the completion finished
    pub finish_reason: Option<String>,

    /// The native finish reason from the provider
    pub native_finish_reason: Option<String>,

    /// Log probabilities for the choice
    pub logprobs: Option<serde_json::Value>,
}

#[cfg(feature = "otel")]
pub mod otel {
    use std::collections::HashMap;

    #[derive(Debug)]
    pub struct SpanWrapper(tracing::Span);

    impl SpanWrapper {
        pub fn get(&self) -> tracing::Span {
            self.0.clone()
        }

        pub fn set(&mut self, span: tracing::Span) {
            self.0 = span;
        }
    }

    impl Default for SpanWrapper {
        fn default() -> Self {
            Self(tracing::Span::current())
        }
    }

    #[derive(Debug, Default)]
    pub struct StreamAggregator {
        pub span: SpanWrapper,
        response_id: Option<String>,
        model: Option<String>,
        provider: Option<String>,
        // Aggregate choices by index to rebuild complete messages
        choices: HashMap<u8, AggregatedChoice>,
        prompt_tokens: Option<u32>,
        completion_tokens: Option<u32>,
    }

    #[derive(Debug, Clone)]
    struct AggregatedChoice {
        /// Aggregated message from all deltas
        message: super::super::Message,
        /// Final finish reason
        finish_reason: Option<String>,
        /// Native finish reason if provided
        native_finish_reason: Option<String>,
    }

    impl Default for AggregatedChoice {
        fn default() -> Self {
            use super::super::{Content, Message, Role};
            Self {
                message: Message::new(Role::Assistant, Content::Simple(String::new())),
                finish_reason: None,
                native_finish_reason: None,
            }
        }
    }

    impl super::super::ChatStream {
        fn span(&self) -> tracing::Span {
            self.aggregated_data.span.get()
        }

        pub fn aggregate_chunk(&mut self, chunk: &super::ChatStreamChunk) {
            use super::super::Content;

            let data = &mut self.aggregated_data;

            // Track response metadata (usually from first chunk)
            if data.response_id.is_none() {
                data.response_id = Some(chunk.id.clone());
            }
            if data.model.is_none() && chunk.model.is_some() {
                data.model = chunk.model.clone();
            }
            if data.provider.is_none() && chunk.provider.is_some() {
                data.provider = chunk.provider.clone();
            }

            // Aggregate each choice by its index
            for choice in &chunk.choices {
                let aggregated = data.choices.entry(choice.index).or_default();

                // Aggregate content from delta
                if let Content::Simple(ref text) = choice.delta.content {
                    if !text.is_empty() {
                        // Append to existing content
                        aggregated.message.content = match &aggregated.message.content {
                            Content::Simple(existing) => Content::Simple(existing.clone() + text),
                            Content::Complex(_) => {
                                aggregated.message.content.clone() + Content::Simple(text.clone())
                            }
                        };
                    }
                } else if let Content::Complex(ref parts) = choice.delta.content {
                    // Handle complex content parts
                    for part in parts {
                        aggregated.message.content =
                            aggregated.message.content.clone().add_part(part.clone());
                    }
                }

                // Update role if provided (usually only in first chunk)
                aggregated.message.role = choice.delta.role.clone();

                // Aggregate reasoning
                if let Some(ref reasoning) = choice.delta.reasoning {
                    aggregated.message.reasoning = Some(
                        aggregated
                            .message
                            .reasoning
                            .as_ref()
                            .map(|r| r.clone() + reasoning)
                            .unwrap_or_else(|| reasoning.clone()),
                    );
                }

                // Aggregate tool calls
                if let Some(ref tool_calls) = choice.delta.tool_calls {
                    if aggregated.message.tool_calls.is_none() {
                        aggregated.message.tool_calls = Some(Vec::new());
                    }
                    if let Some(ref mut agg_tool_calls) = aggregated.message.tool_calls {
                        agg_tool_calls.extend(tool_calls.clone());
                    }
                }

                // Track finish reasons (usually in last chunk for this choice)
                if let Some(ref finish_reason) = choice.finish_reason {
                    aggregated.finish_reason = Some(finish_reason.clone());
                }
                if let Some(ref native_finish_reason) = choice.native_finish_reason {
                    aggregated.native_finish_reason = Some(native_finish_reason.clone());
                }
            }

            // Track usage from final chunk
            if let Some(ref usage) = chunk.usage {
                data.prompt_tokens = Some(usage.prompt_tokens);
                data.completion_tokens = Some(usage.completion_tokens);
            }
        }
    }

    impl Drop for super::super::ChatStream {
        fn drop(&mut self) {
            use super::super::ChatChoice;

            let data = &self.aggregated_data;
            let span = data.span.get();
            let _guard = span.enter();

            // Record response metadata
            if let Some(ref id) = data.response_id {
                span.record("gen_ai.response.id", id);
            }
            if let Some(ref model) = data.model {
                span.record("gen_ai.response.model", model);
            }

            // Emit gen_ai.choice events for each aggregated choice (matching non-streaming behavior)
            let mut finish_reasons = Vec::new();
            for (index, choice) in &data.choices {
                // Create a ChatChoice structure matching the non-streaming version
                let chat_choice = ChatChoice {
                    index: index.clone(),
                    message: choice.message.clone(),
                    finish_reason: choice
                        .finish_reason
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string()),
                };

                // Serialize and emit the choice event
                if let Ok(content) = serde_json::to_string(&chat_choice) {
                    tracing::info!(name: "gen_ai.choice", content);
                }

                // Collect finish reasons for span recording
                if let Some(ref reason) = choice.finish_reason {
                    finish_reasons.push(reason.clone());
                }
            }

            // Record aggregated finish reasons
            if !finish_reasons.is_empty() {
                span.record("gen_ai.response.finish_reasons", finish_reasons.join(","));
            }

            // Record usage statistics
            if let Some(prompt_tokens) = data.prompt_tokens {
                span.record("gen_ai.usage.input_tokens", prompt_tokens);
            }
            if let Some(completion_tokens) = data.completion_tokens {
                span.record("gen_ai.usage.output_tokens", completion_tokens);
            }
        }
    }
}
