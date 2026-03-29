use std::collections::HashMap;

use bon::bon;
use secrecy::SecretString;
use url::Url;

use crate::{
    Result,
    backend::traits::{AsyncRequestBuilder, Backend, Mode, RequestBuilder, SyncRequestBuilder},
    models::Input,
};

const DEFAULT_BASE_URL: &str = open_responses::client::DEFAULT_BASE_URL;

// ── Sync backend ──────────────────────────────────────────────────────────────

/// Backend implementation for any Open Responses-compatible API (sync mode).
pub struct OpenResponsesBackend {
    pub(crate) inner: open_responses::client::Client,
}

#[bon]
impl OpenResponsesBackend {
    #[builder(on(SecretString, into), finish_fn = build_backend)]
    pub fn builder(
        #[builder(field)] headers: HashMap<String, String>,
        #[builder(default = Url::parse(DEFAULT_BASE_URL).expect("default base URL is valid"))]
        base_url: Url,
        api_key: Option<SecretString>,
    ) -> Self {
        let builder = open_responses::client::Client::builder()
            .maybe_api_key(api_key)
            .base_url(base_url);
        let builder = headers
            .into_iter()
            .fold(builder, |b, (k, v)| b.add_header(k, v));
        Self { inner: builder.build() }
    }
}

impl<S: open_responses_backend_builder::State> OpenResponsesBackendBuilder<S> {
    pub fn add_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> OpenResponsesBackend {
        self.build_backend()
    }
}

impl OpenResponsesBackend {
    pub fn new(api_key: impl Into<SecretString>) -> Self {
        Self::builder().api_key(api_key).build_backend()
    }
}

// ── Async backend ─────────────────────────────────────────────────────────────

/// Backend implementation for any Open Responses-compatible API (async mode).
pub struct AsyncOpenResponsesBackend {
    pub(crate) inner: open_responses::client::AsyncClient,
}

#[bon]
impl AsyncOpenResponsesBackend {
    #[builder(on(SecretString, into), finish_fn = build_backend)]
    pub fn builder(
        #[builder(field)] headers: HashMap<String, String>,
        #[builder(default = Url::parse(DEFAULT_BASE_URL).expect("default base URL is valid"))]
        base_url: Url,
        api_key: Option<SecretString>,
    ) -> Self {
        let builder = open_responses::client::AsyncClient::builder()
            .maybe_api_key(api_key)
            .base_url(base_url);
        let builder = headers
            .into_iter()
            .fold(builder, |b, (k, v)| b.add_header(k, v));
        Self { inner: builder.build() }
    }
}

impl<S: async_open_responses_backend_builder::State> AsyncOpenResponsesBackendBuilder<S> {
    pub fn add_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> AsyncOpenResponsesBackend {
        self.build_backend()
    }
}

impl AsyncOpenResponsesBackend {
    pub fn new(api_key: impl Into<SecretString>) -> Self {
        Self::builder().api_key(api_key).build_backend()
    }
}

// ── Mode wrappers ─────────────────────────────────────────────────────────────

pub struct OpenResponsesMode;
pub struct AsyncOpenResponsesMode;

impl Mode for OpenResponsesMode {
    type RequestBuilder<'a> = OpenResponsesRequestBuilder<'a>;
    type Response = open_responses::ResponseResource;
    type StreamResponse = open_responses::client::ResponseStream;
}

impl Mode for AsyncOpenResponsesMode {
    type RequestBuilder<'a> = AsyncOpenResponsesRequestBuilder<'a>;
    type Response = open_responses::ResponseResource;
    type StreamResponse = open_responses::client::AsyncResponseStream;
}

// ── Backend impls ─────────────────────────────────────────────────────────────

impl Backend for OpenResponsesBackend {
    type Mode = OpenResponsesMode;

    fn create_request<'a>(&'a self, input: Input) -> OpenResponsesRequestBuilder<'a> {
        OpenResponsesRequestBuilder {
            inner: self.inner.create_response().input_items(input.0),
        }
    }
}

impl Backend for AsyncOpenResponsesBackend {
    type Mode = AsyncOpenResponsesMode;

    fn create_request<'a>(&'a self, input: Input) -> AsyncOpenResponsesRequestBuilder<'a> {
        AsyncOpenResponsesRequestBuilder {
            inner: self.inner.create_response().input_items(input.0),
        }
    }
}

// ── Request builders ──────────────────────────────────────────────────────────

pub struct OpenResponsesRequestBuilder<'a> {
    inner: open_responses::client::ResponseRequestBuilder<'a>,
}

pub struct AsyncOpenResponsesRequestBuilder<'a> {
    inner: open_responses::client::AsyncResponseRequestBuilder<'a>,
}

macro_rules! impl_request_builder {
    ($ty:ident) => {
        impl<'a> RequestBuilder for $ty<'a> {
            fn model(mut self, model: impl Into<String>) -> Self {
                self.inner = self.inner.model(model);
                self
            }

            fn instructions(mut self, instructions: impl Into<String>) -> Self {
                self.inner = self.inner.instructions(instructions);
                self
            }

            fn previous_response_id(mut self, id: impl Into<String>) -> Self {
                self.inner = self.inner.previous_response_id(id);
                self
            }

            fn tools(mut self, tools: impl IntoIterator<Item = crate::models::Tool>) -> Self {
                self.inner = self.inner.tools(tools.into_iter().map(Into::into));
                self
            }

            fn tool_choice(mut self, choice: open_responses::ToolChoiceParam) -> Self {
                self.inner = self.inner.tool_choice(choice);
                self
            }

            fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
                self.inner = self.inner.metadata(metadata);
                self
            }

            fn text_format(mut self, format: crate::models::Format) -> Self {
                self.inner = self.inner.text(format.into());
                self
            }

            fn temperature(mut self, temperature: f64) -> Self {
                self.inner = self.inner.temperature(temperature);
                self
            }

            fn top_p(mut self, top_p: f64) -> Self {
                self.inner = self.inner.top_p(top_p);
                self
            }

            fn presence_penalty(mut self, presence_penalty: f64) -> Self {
                self.inner = self.inner.presence_penalty(presence_penalty);
                self
            }

            fn frequency_penalty(mut self, frequency_penalty: f64) -> Self {
                self.inner = self.inner.frequency_penalty(frequency_penalty);
                self
            }

            fn parallel_tool_calls(mut self, parallel: bool) -> Self {
                self.inner = self.inner.parallel_tool_calls(parallel);
                self
            }

            fn max_output_tokens(mut self, max: i64) -> Self {
                self.inner = self.inner.max_output_tokens(max);
                self
            }

            fn max_tool_calls(mut self, max: i64) -> Self {
                self.inner = self.inner.max_tool_calls(max);
                self
            }

            fn reasoning(mut self, reasoning: open_responses::ReasoningParam) -> Self {
                self.inner = self.inner.reasoning(reasoning);
                self
            }

            fn truncation(mut self, truncation: open_responses::TruncationEnum) -> Self {
                self.inner = self.inner.truncation(truncation);
                self
            }

            fn include(
                mut self,
                include: impl IntoIterator<Item = open_responses::IncludeEnum>,
            ) -> Self {
                self.inner = self.inner.include(include);
                self
            }

            fn store(mut self, store: bool) -> Self {
                self.inner = self.inner.store(store);
                self
            }

            fn top_logprobs(mut self, top_logprobs: i64) -> Self {
                self.inner = self.inner.top_logprobs(top_logprobs);
                self
            }
        }

        impl<'a> std::fmt::Debug for $ty<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.inner.fmt(f)
            }
        }
    };
}

impl_request_builder!(OpenResponsesRequestBuilder);
impl_request_builder!(AsyncOpenResponsesRequestBuilder);

impl<'a> SyncRequestBuilder for OpenResponsesRequestBuilder<'a> {
    fn send(self) -> Result<open_responses::ResponseResource> {
        self.inner.send()
    }

    fn stream(self) -> Result<open_responses::client::ResponseStream> {
        self.inner.stream()
    }
}

impl<'a> AsyncRequestBuilder for AsyncOpenResponsesRequestBuilder<'a> {
    async fn send(self) -> Result<open_responses::ResponseResource> {
        self.inner.send().await
    }

    async fn stream(self) -> Result<open_responses::client::AsyncResponseStream> {
        self.inner.stream().await
    }
}
