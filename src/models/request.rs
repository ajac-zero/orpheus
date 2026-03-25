use std::collections::HashMap;

use tracing::debug;

use crate::{
    Result,
    client::{
        core::Pool,
        mode::{Async, Mode, Sync},
    },
    constants::RESPONSES_PATH,
    models::{Format, Input, Tool, stream::ResponseStream},
};

/// The serializable request body sent to the API.
#[derive(Debug, serde::Serialize)]
struct ResponseRequest<'a> {
    #[serde(skip)]
    api_key: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,

    input: serde_json::Value,

    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    previous_response_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<open_responses::FunctionToolParam>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<open_responses::ToolChoiceParam>,

    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<open_responses::TextParam>,

    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    parallel_tool_calls: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    max_tool_calls: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<open_responses::ReasoningParam>,

    #[serde(skip_serializing_if = "Option::is_none")]
    truncation: Option<open_responses::TruncationEnum>,

    #[serde(skip_serializing_if = "Option::is_none")]
    include: Option<Vec<open_responses::IncludeEnum>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    store: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    top_logprobs: Option<i64>,
}

/// Public builder for a response request.
pub struct ResponseRequestBuilder<'a, M: Mode> {
    pool: &'a Pool<M>,
    inner: ResponseRequest<'a>,
}

impl<'a, M: Mode> ResponseRequestBuilder<'a, M> {
    pub(crate) fn new(pool: &'a Pool<M>, api_key: Option<&'a str>, input: Input) -> Self {
        let input_value = serde_json::to_value(&input.0).unwrap_or_default();
        Self {
            pool,
            inner: ResponseRequest {
                api_key,
                model: None,
                input: input_value,
                stream: false,
                instructions: None,
                previous_response_id: None,
                tools: None,
                tool_choice: None,
                metadata: None,
                text: None,
                temperature: None,
                top_p: None,
                presence_penalty: None,
                frequency_penalty: None,
                parallel_tool_calls: None,
                max_output_tokens: None,
                max_tool_calls: None,
                reasoning: None,
                truncation: None,
                include: None,
                store: None,
                top_logprobs: None,
            },
        }
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.inner.model = Some(model.into());
        self
    }

    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.inner.instructions = Some(instructions.into());
        self
    }

    pub fn previous_response_id(mut self, id: impl Into<String>) -> Self {
        self.inner.previous_response_id = Some(id.into());
        self
    }

    pub fn tools(mut self, tools: impl IntoIterator<Item = Tool>) -> Self {
        self.inner.tools = Some(tools.into_iter().map(Into::into).collect());
        self
    }

    pub fn tool_choice(mut self, choice: open_responses::ToolChoiceParam) -> Self {
        self.inner.tool_choice = Some(choice);
        self
    }

    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.inner.metadata = Some(metadata);
        self
    }

    pub fn text_format(mut self, format: Format) -> Self {
        self.inner.text = Some(format.into());
        self
    }

    pub fn temperature(mut self, temperature: f64) -> Self {
        self.inner.temperature = Some(temperature);
        self
    }

    pub fn top_p(mut self, top_p: f64) -> Self {
        self.inner.top_p = Some(top_p);
        self
    }

    pub fn presence_penalty(mut self, presence_penalty: f64) -> Self {
        self.inner.presence_penalty = Some(presence_penalty);
        self
    }

    pub fn frequency_penalty(mut self, frequency_penalty: f64) -> Self {
        self.inner.frequency_penalty = Some(frequency_penalty);
        self
    }

    pub fn parallel_tool_calls(mut self, parallel: bool) -> Self {
        self.inner.parallel_tool_calls = Some(parallel);
        self
    }

    pub fn max_output_tokens(mut self, max: i64) -> Self {
        self.inner.max_output_tokens = Some(max);
        self
    }

    pub fn max_tool_calls(mut self, max: i64) -> Self {
        self.inner.max_tool_calls = Some(max);
        self
    }

    pub fn reasoning(mut self, reasoning: open_responses::ReasoningParam) -> Self {
        self.inner.reasoning = Some(reasoning);
        self
    }

    pub fn truncation(mut self, truncation: open_responses::TruncationEnum) -> Self {
        self.inner.truncation = Some(truncation);
        self
    }

    pub fn include(
        mut self,
        include: impl IntoIterator<Item = open_responses::IncludeEnum>,
    ) -> Self {
        self.inner.include = Some(include.into_iter().collect());
        self
    }

    pub fn store(mut self, store: bool) -> Self {
        self.inner.store = Some(store);
        self
    }

    pub fn top_logprobs(mut self, top_logprobs: i64) -> Self {
        self.inner.top_logprobs = Some(top_logprobs);
        self
    }
}

impl<'a> ResponseRequestBuilder<'a, Sync> {
    /// Send the request and return a complete response.
    pub fn send(mut self) -> Result<open_responses::ResponseResource> {
        let mut handler = self.pool.get().expect("Has handler");
        self.inner.stream = false;
        let token = self.inner.api_key;
        let body = &self.inner;
        debug!(request_body = ?body);

        let response = handler
            .execute()
            .segments(&RESPONSES_PATH)
            .payload(body)
            .maybe_token(token)
            .call()?;

        let resource = response.json::<open_responses::ResponseResource>()?;
        debug!(response = ?resource);

        Ok(resource)
    }

    /// Send the request and return a streaming response.
    pub fn stream(mut self) -> Result<ResponseStream<Sync>> {
        let mut handler = self.pool.get().expect("Has handler");
        self.inner.stream = true;
        let token = self.inner.api_key;
        let body = &self.inner;

        let response = handler
            .execute()
            .segments(&RESPONSES_PATH)
            .payload(body)
            .maybe_token(token)
            .call()?;

        Ok(ResponseStream::new(response, handler.mode.clone()))
    }
}

impl<'a> ResponseRequestBuilder<'a, Async> {
    /// Asynchronously send the request and return a complete response.
    pub async fn send(mut self) -> Result<open_responses::ResponseResource> {
        let mut handler = self.pool.get().await.expect("Has handler");
        self.inner.stream = false;
        let token = self.inner.api_key;
        let body = &self.inner;
        debug!(request_body = ?body);

        let response = handler
            .execute()
            .segments(&RESPONSES_PATH)
            .payload(body)
            .maybe_token(token)
            .call()
            .await?;

        let resource = response.json::<open_responses::ResponseResource>().await?;
        debug!(response = ?resource);

        Ok(resource)
    }

    /// Asynchronously send the request and return a streaming response.
    pub async fn stream(mut self) -> Result<ResponseStream<Async>> {
        let mut handler = self.pool.get().await.expect("Has handler");
        self.inner.stream = true;
        let token = self.inner.api_key;
        let body = &self.inner;

        let response = handler
            .execute()
            .segments(&RESPONSES_PATH)
            .payload(body)
            .maybe_token(token)
            .call()
            .await?;

        Ok(ResponseStream::new(response, handler.mode.clone()))
    }
}

impl<M: Mode> std::fmt::Debug for ResponseRequestBuilder<'_, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResponseRequestBuilder")
            .field("inner", &self.inner)
            .finish()
    }
}
