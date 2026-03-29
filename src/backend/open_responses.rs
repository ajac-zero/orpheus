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

/// Backend implementation for any Open Responses-compatible API (e.g. OpenRouter, OpenAI).
pub struct OpenResponsesBackend<M: open_responses::client::Mode> {
    pub(crate) inner: open_responses::client::ClientCore<M>,
}

impl<M: open_responses::client::Mode> OpenResponsesBackend<M> {
    pub fn new(api_key: impl Into<SecretString>) -> Self {
        Self::builder().api_key(api_key).build_backend()
    }
}

#[bon]
impl<M: open_responses::client::Mode> OpenResponsesBackend<M> {
    #[builder(on(SecretString, into), finish_fn = build_backend)]
    pub fn builder(
        #[builder(field)] headers: HashMap<String, String>,
        #[builder(default = Url::parse(DEFAULT_BASE_URL).expect("default base URL is valid"))]
        base_url: Url,
        api_key: Option<SecretString>,
    ) -> Self {
        let builder = open_responses::client::ClientCore::<M>::builder()
            .maybe_api_key(api_key)
            .base_url(base_url);
        let builder = headers.into_iter().fold(builder, |builder, (key, value)| {
            builder.add_header(key, value)
        });

        Self {
            inner: builder.build(),
        }
    }
}

impl<M: open_responses::client::Mode, S: open_responses_backend_builder::State>
    OpenResponsesBackendBuilder<M, S>
{
    pub fn add_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> OpenResponsesBackend<M> {
        self.build_backend()
    }
}

impl Backend for OpenResponsesBackend<open_responses::client::Sync> {
    type Mode = OpenResponsesMode<open_responses::client::Sync>;

    fn create_request<'a>(&'a self, input: Input) -> OpenResponsesRequestBuilder<'a, open_responses::client::Sync> {
        OpenResponsesRequestBuilder {
            inner: self.inner.create_response().input_items(input.0),
        }
    }
}

impl Backend for OpenResponsesBackend<open_responses::client::Async> {
    type Mode = OpenResponsesMode<open_responses::client::Async>;

    fn create_request<'a>(&'a self, input: Input) -> OpenResponsesRequestBuilder<'a, open_responses::client::Async> {
        OpenResponsesRequestBuilder {
            inner: self.inner.create_response().input_items(input.0),
        }
    }
}

/// Mode wrapper for Open Responses backends.
pub struct OpenResponsesMode<M: open_responses::client::Mode>(std::marker::PhantomData<M>);

impl Mode for OpenResponsesMode<open_responses::client::Sync> {
    type RequestBuilder<'a> = OpenResponsesRequestBuilder<'a, open_responses::client::Sync>;
    type Response = open_responses::ResponseResource;
    type StreamResponse = open_responses::client::ResponseStream;
}

impl Mode for OpenResponsesMode<open_responses::client::Async> {
    type RequestBuilder<'a> = OpenResponsesRequestBuilder<'a, open_responses::client::Async>;
    type Response = open_responses::ResponseResource;
    type StreamResponse = open_responses::client::ResponseStream;
}

/// Request builder for Open Responses-compatible backends.
pub struct OpenResponsesRequestBuilder<'a, M: open_responses::client::Mode> {
    inner: open_responses::client::ResponseRequestBuilder<'a, M>,
}

impl<'a, M: open_responses::client::Mode> RequestBuilder for OpenResponsesRequestBuilder<'a, M> {
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

impl<'a> SyncRequestBuilder for OpenResponsesRequestBuilder<'a, open_responses::client::Sync> {
    fn send(self) -> Result<open_responses::ResponseResource> {
        self.inner.send()
    }

    fn stream(self) -> Result<open_responses::client::ResponseStream> {
        self.inner.stream()
    }
}

impl<'a> AsyncRequestBuilder for OpenResponsesRequestBuilder<'a, open_responses::client::Async> {
    fn send(self) -> impl std::future::Future<Output = Result<open_responses::ResponseResource>> {
        self.inner.send()
    }

    fn stream(
        self,
    ) -> impl std::future::Future<Output = Result<open_responses::client::ResponseStream>> {
        self.inner.stream()
    }
}

impl<M: open_responses::client::Mode> std::fmt::Debug for OpenResponsesRequestBuilder<'_, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}
