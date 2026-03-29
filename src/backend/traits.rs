use std::collections::HashMap;

use crate::{models::Input, Result};

/// Mode trait for sync/async operations.
pub trait Mode: 'static + Send + std::marker::Sync {
    type RequestBuilder<'a>: RequestBuilder;
    type Response: Response;
    type StreamResponse: StreamResponse;
}



/// Core backend trait that all providers must implement.
pub trait Backend: Sized {
    type Mode: Mode;

    fn create_request<'a>(&'a self, input: Input) -> <Self::Mode as Mode>::RequestBuilder<'a>;
}

/// Request builder trait for configuring API requests.
pub trait RequestBuilder: Sized {
    fn model(self, model: impl Into<String>) -> Self;
    fn instructions(self, instructions: impl Into<String>) -> Self;
    fn previous_response_id(self, id: impl Into<String>) -> Self;
    fn tools(self, tools: impl IntoIterator<Item = crate::models::Tool>) -> Self;
    fn tool_choice(self, choice: open_responses::ToolChoiceParam) -> Self;
    fn metadata(self, metadata: HashMap<String, String>) -> Self;
    fn text_format(self, format: crate::models::Format) -> Self;
    fn temperature(self, temperature: f64) -> Self;
    fn top_p(self, top_p: f64) -> Self;
    fn presence_penalty(self, presence_penalty: f64) -> Self;
    fn frequency_penalty(self, frequency_penalty: f64) -> Self;
    fn parallel_tool_calls(self, parallel: bool) -> Self;
    fn max_output_tokens(self, max: i64) -> Self;
    fn max_tool_calls(self, max: i64) -> Self;
    fn reasoning(self, reasoning: open_responses::ReasoningParam) -> Self;
    fn truncation(self, truncation: open_responses::TruncationEnum) -> Self;
    fn include(self, include: impl IntoIterator<Item = open_responses::IncludeEnum>) -> Self;
    fn store(self, store: bool) -> Self;
    fn top_logprobs(self, top_logprobs: i64) -> Self;
}

/// Sync request builder with send/stream methods.
pub trait SyncRequestBuilder: RequestBuilder {
    fn send(self) -> Result<open_responses::ResponseResource>;
    fn stream(self) -> Result<open_responses::client::ResponseStream>;
}

/// Async request builder with async send/stream methods.
pub trait AsyncRequestBuilder: RequestBuilder {
    fn send(self) -> impl std::future::Future<Output = Result<open_responses::ResponseResource>>;
    fn stream(
        self,
    ) -> impl std::future::Future<Output = Result<open_responses::client::AsyncResponseStream>>;
}

/// Response marker trait for abstracting over different response types.
pub trait Response {}

/// Stream response marker trait for abstracting over different streaming responses.
pub trait StreamResponse {}

impl Response for open_responses::ResponseResource {}

impl StreamResponse for open_responses::client::ResponseStream {}

impl StreamResponse for open_responses::client::AsyncResponseStream {}
