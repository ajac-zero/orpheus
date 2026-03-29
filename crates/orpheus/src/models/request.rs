use std::collections::HashMap;

use crate::{
    Result,
    client::{AsyncOrpheus, Orpheus},
    models::{Format, Input, Tool},
};

/// Public builder for a blocking response request.
pub struct ResponseRequestBuilder<'a> {
    inner: open_responses::client::ResponseRequestBuilder<'a>,
}

/// Public builder for an async response request.
pub struct AsyncResponseRequestBuilder<'a> {
    inner: open_responses::client::AsyncResponseRequestBuilder<'a>,
}

macro_rules! impl_request_builder_methods {
    ($builder:ident) => {
        impl<'a> $builder<'a> {
            pub fn model(mut self, model: impl Into<String>) -> Self {
                self.inner = self.inner.model(model);
                self
            }

            pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
                self.inner = self.inner.instructions(instructions);
                self
            }

            pub fn previous_response_id(mut self, id: impl Into<String>) -> Self {
                self.inner = self.inner.previous_response_id(id);
                self
            }

            pub fn tools(mut self, tools: impl IntoIterator<Item = Tool>) -> Self {
                self.inner = self.inner.tools(tools.into_iter().map(Into::into));
                self
            }

            pub fn tool_choice(mut self, choice: open_responses::ToolChoiceParam) -> Self {
                self.inner = self.inner.tool_choice(choice);
                self
            }

            pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
                self.inner = self.inner.metadata(metadata);
                self
            }

            pub fn text_format(mut self, format: Format) -> Self {
                self.inner = self.inner.text(format.into());
                self
            }

            pub fn temperature(mut self, temperature: f64) -> Self {
                self.inner = self.inner.temperature(temperature);
                self
            }

            pub fn top_p(mut self, top_p: f64) -> Self {
                self.inner = self.inner.top_p(top_p);
                self
            }

            pub fn presence_penalty(mut self, presence_penalty: f64) -> Self {
                self.inner = self.inner.presence_penalty(presence_penalty);
                self
            }

            pub fn frequency_penalty(mut self, frequency_penalty: f64) -> Self {
                self.inner = self.inner.frequency_penalty(frequency_penalty);
                self
            }

            pub fn parallel_tool_calls(mut self, parallel: bool) -> Self {
                self.inner = self.inner.parallel_tool_calls(parallel);
                self
            }

            pub fn max_output_tokens(mut self, max: i64) -> Self {
                self.inner = self.inner.max_output_tokens(max);
                self
            }

            pub fn max_tool_calls(mut self, max: i64) -> Self {
                self.inner = self.inner.max_tool_calls(max);
                self
            }

            pub fn reasoning(mut self, reasoning: open_responses::ReasoningParam) -> Self {
                self.inner = self.inner.reasoning(reasoning);
                self
            }

            pub fn truncation(mut self, truncation: open_responses::TruncationEnum) -> Self {
                self.inner = self.inner.truncation(truncation);
                self
            }

            pub fn include(
                mut self,
                include: impl IntoIterator<Item = open_responses::IncludeEnum>,
            ) -> Self {
                self.inner = self.inner.include(include);
                self
            }

            pub fn store(mut self, store: bool) -> Self {
                self.inner = self.inner.store(store);
                self
            }

            pub fn top_logprobs(mut self, top_logprobs: i64) -> Self {
                self.inner = self.inner.top_logprobs(top_logprobs);
                self
            }
        }
    };
}

impl_request_builder_methods!(ResponseRequestBuilder);
impl_request_builder_methods!(AsyncResponseRequestBuilder);

impl<'a> ResponseRequestBuilder<'a> {
    pub(crate) fn new(core: &'a Orpheus, input: Input) -> Self {
        Self {
            inner: core.inner.create_response().input_items(input.0),
        }
    }

    /// Send the request and return a complete response.
    pub fn send(self) -> Result<open_responses::ResponseResource> {
        self.inner.send()
    }

    /// Send the request and return a streaming response.
    pub fn stream(self) -> Result<open_responses::client::ResponseStream> {
        self.inner.stream()
    }
}

impl<'a> AsyncResponseRequestBuilder<'a> {
    pub(crate) fn new(core: &'a AsyncOrpheus, input: Input) -> Self {
        Self {
            inner: core.inner.create_response().input_items(input.0),
        }
    }

    /// Asynchronously send the request and return a complete response.
    pub async fn send(self) -> Result<open_responses::ResponseResource> {
        self.inner.send().await
    }

    /// Asynchronously send the request and return a streaming response.
    pub async fn stream(self) -> Result<open_responses::client::AsyncResponseStream> {
        self.inner.stream().await
    }
}

impl std::fmt::Debug for ResponseRequestBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::fmt::Debug for AsyncResponseRequestBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
