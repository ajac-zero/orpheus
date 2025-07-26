use std::collections::HashMap;

use super::main::AsyncOrpheus;
use crate::{
    Error, Result,
    constants::*,
    models::{
        chat::*,
        common::{provider::*, reasoning::*, usage::*},
    },
};

#[serde_with::skip_serializing_none]
#[derive(Debug, serde::Serialize, bon::Builder)]
#[builder(on(String, into))]
pub struct ChatRequest {
    #[serde(skip)]
    #[builder(start_fn)]
    handler: Option<AsyncOrpheus>,

    /// List of messages in the conversation
    #[builder(start_fn, into)]
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

impl<S: chat_request_builder::State> ChatRequestBuilder<S> {
    pub async fn send(mut self) -> Result<ChatCompletion>
    where
        S: chat_request_builder::IsComplete,
    {
        let handler = self.handler.take().unwrap();

        self.stream = Some(false);
        let body = self.build();

        let response = handler.execute(CHAT_COMPLETION_PATH, body).await?;

        let chat_completion = response
            .json::<ChatCompletion>()
            .await
            .map_err(Error::http)?;

        Ok(chat_completion)
    }

    pub async fn stream(mut self) -> Result<AsyncStream>
    where
        S: chat_request_builder::IsComplete,
    {
        let handler = self.handler.take().unwrap();

        self.stream = Some(true);
        let body = self.build();

        let response = handler.execute(CHAT_COMPLETION_PATH, body).await?;

        Ok(response.into())
    }

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

impl AsyncOrpheus {
    pub fn chat(&self, messages: impl Into<ChatMessages>) -> ChatRequestBuilder {
        let handler = self.clone();
        ChatRequest::builder(Some(handler), messages)
    }
}
