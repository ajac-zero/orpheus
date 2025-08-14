use std::collections::HashMap;

#[cfg(feature = "otel")]
use tracing::Span;
use tracing::debug;

use crate::{
    Error, Result,
    models::{
        chat::*,
        common::{
            handler::{AsyncHandler, Handler},
            mode::{Async, Mode, Sync},
            provider::*,
            reasoning::*,
            usage::*,
        },
    },
};

#[serde_with::skip_serializing_none]
#[derive(Debug, serde::Serialize, bon::Builder)]
#[builder(on(String, into), derive(Debug))]
pub struct ChatRequest<M: Mode> {
    #[cfg(feature = "otel")]
    #[serde(skip)]
    #[builder(start_fn)]
    pub span: Span,

    #[serde(skip)]
    #[builder(start_fn)]
    handler: Option<ChatHandler<M>>,

    /// List of messages in the conversation
    #[builder(into, start_fn)]
    pub messages: ChatMessages,

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

    #[builder(into)]
    pub response_format: Option<ResponseFormat>,

    /// Alternate list of models for routing overrides.
    #[builder(name = "fallbacks", with = |models: impl IntoIterator<Item: Into<String>>| models.into_iter().map(Into::into).collect())]
    pub models: Option<Vec<String>>,

    #[builder(into)]
    pub tools: Option<Tools>,

    pub plugins: Option<Vec<Plugin>>,

    /// Whether to include usage information in the response
    pub usage: Option<UsageConfig>,

    /// List of prompt transforms (OpenRouter-only).
    pub transforms: Option<Vec<String>>,

    /// Maximum number of tokens (range: [1, context_length)).
    pub max_tokens: Option<i32>,

    /// Sampling temperature (range: [0, 2]).
    pub temperature: Option<f64>,

    /// Seed for deterministic outputs.
    pub seed: Option<i32>,

    /// Top-p sampling value (range: (0, 1]).
    pub top_p: Option<f64>,

    /// Top-k sampling value (range: [1, Infinity)).
    pub top_k: Option<i32>,

    /// Frequency penalty (range: [-2, 2]).
    pub frequency_penalty: Option<f64>,

    /// Presence penalty (range: [-2, 2]).
    pub presence_penalty: Option<f64>,

    /// Repetition penalty (range: (0, 2]).
    pub repetition_penalty: Option<f64>,

    /// Mapping of token IDs to bias values.
    pub logit_bias: Option<HashMap<String, f64>>,

    /// Number of top log probabilities to return.
    pub top_logprobs: Option<i32>,

    /// Minimum probability threshold (range: [0, 1]).
    pub min_p: Option<f64>,

    /// Alternate top sampling parameter (range: [0, 1]).
    pub top_a: Option<f64>,

    /// A stable identifier for your end-users. Used to help detect and prevent abuse.
    pub user: Option<String>,
}

impl<M: Mode, S: chat_request_builder::State> ChatRequestBuilder<M, S> {
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
    pub fn send(mut self) -> Result<ChatCompletion> {
        #[cfg(feature = "otel")]
        let span = self.span.clone();

        let handler = self.handler.take().expect("Has handler");

        self.stream = Some(false);
        let body = self.build();
        debug!(chat_request_body = ?body);

        let response = handler.execute(body)?;

        let chat_completion = response.json::<ChatCompletion>().map_err(Error::http)?;
        debug!(chat_completion_response = ?chat_completion);

        #[cfg(feature = "otel")]
        {
            let _guard = span.enter();

            for choice in chat_completion.choices.iter() {
                let content = serde_json::to_string(choice).expect("serializable");
                tracing::info!(name: "gen_ai.choice", content);
            }

            span.record("gen_ai.response.id", &chat_completion.id);
            span.record("gen_ai.response.model", &chat_completion.model);

            let mut finish_reasons = Vec::new();
            for choice in &chat_completion.choices {
                finish_reasons.push(choice.finish_reason.clone());
            }
            span.record("gen_ai.response.finish_reasons", finish_reasons.join(","));

            span.record(
                "gen_ai.usage.input_tokens",
                &chat_completion.usage.prompt_tokens,
            );
            span.record(
                "gen_ai.usage.output_tokens",
                &chat_completion.usage.completion_tokens,
            );
        }

        Ok(chat_completion)
    }

    pub fn stream(mut self) -> Result<ChatStream> {
        #[cfg(feature = "otel")]
        let span = self.span.clone();

        let handler = self.handler.take().expect("Has handler");

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
    pub async fn send(mut self) -> Result<ChatCompletion> {
        #[cfg(feature = "otel")]
        let span = self.span.clone();

        let handler = self.handler.take().expect("Has handler");

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
        {
            let _guard = span.enter();

            for choice in chat_completion.choices.iter() {
                let content = serde_json::to_string(choice).expect("serializable");
                tracing::info!(name: "gen_ai.choice", content);
            }

            span.record("gen_ai.response.id", &chat_completion.id);
            span.record("gen_ai.response.model", &chat_completion.model);

            let mut finish_reasons = Vec::new();
            for choice in &chat_completion.choices {
                finish_reasons.push(choice.finish_reason.clone());
            }
            span.record("gen_ai.response.finish_reasons", finish_reasons.join(","));

            span.record(
                "gen_ai.usage.input_tokens",
                &chat_completion.usage.prompt_tokens,
            );
            span.record(
                "gen_ai.usage.output_tokens",
                &chat_completion.usage.completion_tokens,
            );
        }

        Ok(chat_completion)
    }

    pub async fn stream(mut self) -> Result<AsyncStream> {
        #[cfg(feature = "otel")]
        let span = self.span.clone();

        let handler = self.handler.take().expect("Has handler");

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
