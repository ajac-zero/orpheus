use std::collections::HashMap;

#[cfg(feature = "otel")]
use tracing::Span;
use tracing::debug;

use crate::{
    Error, Result,
    client::core::{Async, AsyncExecutor, Executor, Mode, Sync},
    models::{
        Format, Plugin, ProviderPreferences, ReasoningConfig, UsageConfig,
        chat::{AsyncStream, ChatCompletion, ChatHandler, ChatStream, History, Tool},
        common::{
            ProviderPreferencesBuilder, ReasoningConfigBuilder, provider_preferences_builder,
            reasoning_config_builder,
        },
    },
};

/// Core request structure for chat completion API calls.
///
/// This module contains the `ChatRequest` struct which represents all possible
/// parameters that can be sent to a chat completion API. It supports both
/// synchronous and asynchronous operations through the `Mode` generic parameter.

#[serde_with::skip_serializing_none]
#[derive(Debug, serde::Serialize, bon::Builder)]
#[builder(
    on(String, into),
    derive(Debug),
    builder_type(vis = "pub", doc {
        /// Builder to set the parameters of a chat request
    })
)]
pub(crate) struct ChatRequest<M: Mode> {
    #[cfg(feature = "otel")]
    #[serde(skip)]
    #[builder(start_fn)]
    pub span: Span,

    #[serde(skip)]
    #[builder(start_fn)]
    handler: Option<ChatHandler<M>>,

    /// List of messages in the conversation
    #[builder(into, start_fn)]
    pub messages: History,

    /// Enable streaming of results. Defaults to false
    #[builder(field)]
    pub stream: Option<bool>,

    /// Preferences for provider routing.
    #[builder(field)]
    pub provider: Option<ProviderPreferences>,

    /// Configuration for model reasoning/thinking tokens
    #[builder(field)]
    pub reasoning: Option<ReasoningConfig>,

    /// The model ID to use. If unspecified, the user's default is used.
    pub model: Option<String>,

    /// Optional structured output format specification.
    #[builder(into)]
    pub response_format: Option<Format>,

    /// Alternate list of models for routing overrides.
    #[builder(name = "fallbacks", with = |models: impl IntoIterator<Item: Into<String>>| models.into_iter().map(Into::into).collect())]
    pub models: Option<Vec<String>>,

    /// Optional collection of tools (functions) the model can call.
    #[builder(into)]
    pub tools: Option<Vec<Tool>>,

    #[builder(into)]
    pub plugins: Option<Vec<Plugin>>,

    /// Whether to include usage information in the response
    pub usage: Option<UsageConfig>,

    /// List of prompt transforms (OpenRouter-only).
    pub transforms: Option<Vec<String>>,

    /// Maximum number of tokens.
    pub max_tokens: Option<i32>,

    /// Sampling temperature.
    pub temperature: Option<f64>,

    /// Seed for deterministic outputs.
    pub seed: Option<i32>,

    /// Top-p sampling value.
    pub top_p: Option<f64>,

    /// Top-k sampling value.
    pub top_k: Option<i32>,

    /// Frequency penalty.
    pub frequency_penalty: Option<f64>,

    /// Presence penalty.
    pub presence_penalty: Option<f64>,

    /// Repetition penalty.
    pub repetition_penalty: Option<f64>,

    /// Mapping of token IDs to bias values.
    pub logit_bias: Option<HashMap<String, f64>>,

    /// Number of top log probabilities to return.
    pub top_logprobs: Option<i32>,

    /// Minimum probability threshold.
    pub min_p: Option<f64>,

    /// Alternate top sampling parameter.
    pub top_a: Option<f64>,

    /// A stable identifier for your end-users. Used to help detect and prevent abuse.
    pub user: Option<String>,
}

impl<M: Mode, S: chat_request_builder::State> ChatRequestBuilder<M, S> {
    /// Sets provider routing preferences for model selection.
    pub fn preferences(mut self, preferences: ProviderPreferences) -> Self {
        self.provider = Some(preferences);
        self
    }

    pub fn with_preferences<F, C>(mut self, build_preferences: F) -> Self
    where
        F: FnOnce(ProviderPreferencesBuilder) -> ProviderPreferencesBuilder<C>,
        C: provider_preferences_builder::IsComplete,
    {
        let builder = ProviderPreferences::builder();
        let preferences = build_preferences(builder).build();
        self.provider = Some(preferences);
        self
    }

    pub fn reasoning(mut self, config: ReasoningConfig) -> Self {
        self.reasoning = Some(config);
        self
    }

    pub fn with_reasoning<F, C>(mut self, build_reasoning: F) -> Self
    where
        F: FnOnce(ReasoningConfigBuilder) -> ReasoningConfigBuilder<C>,
        C: reasoning_config_builder::IsComplete,
    {
        let builder = ReasoningConfig::builder();
        let config = build_reasoning(builder).build();
        self.reasoning = Some(config);
        self
    }
}

impl<S: chat_request_builder::State> ChatRequestBuilder<Sync, S>
where
    S: chat_request_builder::IsComplete,
{
    /// Sends the chat request and returns a complete response.
    pub fn send(mut self) -> Result<ChatCompletion> {
        #[cfg(feature = "otel")]
        let span = self.span.clone();

        let handler = self.handler.take().expect("Has handler");

        // Disable streaming for complete response
        self.stream = Some(false);
        let body = self.build();
        debug!(chat_request_body = ?body);

        let response = handler.execute(body)?;

        let chat_completion = response.json::<ChatCompletion>().map_err(Error::http)?;
        debug!(chat_completion_response = ?chat_completion);

        #[cfg(feature = "otel")]
        crate::otel::record_completion(span, &chat_completion);

        Ok(chat_completion)
    }

    /// Sends the chat request and returns a streaming response.
    pub fn stream(mut self) -> Result<ChatStream> {
        #[cfg(feature = "otel")]
        let span = self.span.clone();

        let handler = self.handler.take().expect("Has handler");

        // Enable streaming for real-time response
        self.stream = Some(true);
        let body = self.build();
        let response = handler.execute(body)?;

        #[allow(unused_mut)]
        let mut stream = ChatStream::new(response);

        #[cfg(feature = "otel")]
        stream.aggregator.set_span(span);

        Ok(stream)
    }
}

impl<S: chat_request_builder::State> ChatRequestBuilder<Async, S>
where
    S: chat_request_builder::IsComplete,
{
    /// Asynchronously sends the chat request and returns a complete response.
    pub async fn send(mut self) -> Result<ChatCompletion> {
        #[cfg(feature = "otel")]
        let span = self.span.clone();

        let handler = self.handler.take().expect("Has handler");

        // Disable streaming for complete response
        self.stream = Some(false);
        let body = self.build();
        debug!(chat_request_body = ?body);

        let response = handler.execute(body).await?;

        let chat_completion = response
            .json::<ChatCompletion>()
            .await
            .map_err(Error::http)?;
        debug!(chat_completion_response = ?chat_completion);

        #[cfg(feature = "otel")]
        crate::otel::record_completion(span, &chat_completion);

        Ok(chat_completion)
    }

    /// Asynchronously sends the chat request and returns a streaming response.
    pub async fn stream(mut self) -> Result<AsyncStream> {
        #[cfg(feature = "otel")]
        let span = self.span.clone();

        let handler = self.handler.take().expect("Has handler");

        // Enable streaming for real-time response
        self.stream = Some(true);
        let body = self.build();

        let response = handler.execute(body).await?;

        #[allow(unused_mut)]
        let mut stream = AsyncStream::new(response);

        #[cfg(feature = "otel")]
        stream.aggregator.set_span(span);

        Ok(stream)
    }
}
