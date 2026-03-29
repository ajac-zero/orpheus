use std::collections::HashMap;

use crate::{
    Result,
    backend::{Backend, RequestBuilder as BackendRequestBuilder, SyncRequestBuilder, AsyncRequestBuilder},
    client::OrpheusCore,
    models::{Format, Input, Tool},
};

/// Public builder for a response request.
pub struct ResponseRequestBuilder<'a, B: Backend> {
    inner: <B::Mode as crate::backend::Mode>::RequestBuilder<'a>,
}

impl<'a, B: Backend> ResponseRequestBuilder<'a, B> {
    pub(crate) fn new(core: &'a OrpheusCore<B>, input: Input) -> Self {
        Self {
            inner: core.backend.create_request(input),
        }
    }

    pub fn model(self, model: impl Into<String>) -> Self {
        Self {
            inner: self.inner.model(model),
        }
    }

    pub fn instructions(self, instructions: impl Into<String>) -> Self {
        Self {
            inner: self.inner.instructions(instructions),
        }
    }

    pub fn previous_response_id(self, id: impl Into<String>) -> Self {
        Self {
            inner: self.inner.previous_response_id(id),
        }
    }

    pub fn tools(self, tools: impl IntoIterator<Item = Tool>) -> Self {
        Self {
            inner: self.inner.tools(tools),
        }
    }

    pub fn tool_choice(self, choice: open_responses::ToolChoiceParam) -> Self {
        Self {
            inner: self.inner.tool_choice(choice),
        }
    }

    pub fn metadata(self, metadata: HashMap<String, String>) -> Self {
        Self {
            inner: self.inner.metadata(metadata),
        }
    }

    pub fn text_format(self, format: Format) -> Self {
        Self {
            inner: self.inner.text_format(format),
        }
    }

    pub fn temperature(self, temperature: f64) -> Self {
        Self {
            inner: self.inner.temperature(temperature),
        }
    }

    pub fn top_p(self, top_p: f64) -> Self {
        Self {
            inner: self.inner.top_p(top_p),
        }
    }

    pub fn presence_penalty(self, presence_penalty: f64) -> Self {
        Self {
            inner: self.inner.presence_penalty(presence_penalty),
        }
    }

    pub fn frequency_penalty(self, frequency_penalty: f64) -> Self {
        Self {
            inner: self.inner.frequency_penalty(frequency_penalty),
        }
    }

    pub fn parallel_tool_calls(self, parallel: bool) -> Self {
        Self {
            inner: self.inner.parallel_tool_calls(parallel),
        }
    }

    pub fn max_output_tokens(self, max: i64) -> Self {
        Self {
            inner: self.inner.max_output_tokens(max),
        }
    }

    pub fn max_tool_calls(self, max: i64) -> Self {
        Self {
            inner: self.inner.max_tool_calls(max),
        }
    }

    pub fn reasoning(self, reasoning: open_responses::ReasoningParam) -> Self {
        Self {
            inner: self.inner.reasoning(reasoning),
        }
    }

    pub fn truncation(self, truncation: open_responses::TruncationEnum) -> Self {
        Self {
            inner: self.inner.truncation(truncation),
        }
    }

    pub fn include(
        self,
        include: impl IntoIterator<Item = open_responses::IncludeEnum>,
    ) -> Self {
        Self {
            inner: self.inner.include(include),
        }
    }

    pub fn store(self, store: bool) -> Self {
        Self {
            inner: self.inner.store(store),
        }
    }

    pub fn top_logprobs(self, top_logprobs: i64) -> Self {
        Self {
            inner: self.inner.top_logprobs(top_logprobs),
        }
    }
}

impl<'a> ResponseRequestBuilder<'a, crate::backend::OpenResponsesBackend<open_responses::client::Sync>> {
    /// Send the request and return a complete response.
    pub fn send(self) -> Result<open_responses::ResponseResource> {
        self.inner.send()
    }

    /// Send the request and return a streaming response.
    pub fn stream(self) -> Result<open_responses::client::ResponseStream> {
        self.inner.stream()
    }
}

impl<'a> ResponseRequestBuilder<'a, crate::backend::OpenResponsesBackend<open_responses::client::Async>> {
    /// Asynchronously send the request and return a complete response.
    pub async fn send(self) -> Result<open_responses::ResponseResource> {
        self.inner.send().await
    }

    /// Asynchronously send the request and return a streaming response.
    pub async fn stream(self) -> Result<open_responses::client::ResponseStream> {
        self.inner.stream().await
    }
}

impl<B: Backend> std::fmt::Debug for ResponseRequestBuilder<'_, B>
where
    for<'a> <B::Mode as crate::backend::Mode>::RequestBuilder<'a>: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
