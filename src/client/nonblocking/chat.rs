use std::collections::HashMap;

#[cfg(feature = "otel")]
use tracing::{Span, field::Empty, info, instrument};

use super::main::{AsyncHandler, AsyncOrpheus};
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
    #[cfg(feature = "otel")]
    #[serde(skip)]
    #[builder(start_fn)]
    span: Span,

    #[serde(skip)]
    #[builder(start_fn)]
    handler: Option<ChatHandler>,

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
        #[cfg(feature = "otel")]
        let span = self.span.clone();

        let handler = self.handler.take().unwrap();

        self.stream = Some(false);
        let body = self.build();

        let response = handler.execute(body).await?;

        let chat_completion = response
            .json::<ChatCompletion>()
            .await
            .map_err(Error::http)?;

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

    pub async fn stream(mut self) -> Result<AsyncStream>
    where
        S: chat_request_builder::IsComplete,
    {
        #[cfg(feature = "otel")]
        let span = self.span.clone();

        let handler = self.handler.take().unwrap();

        self.stream = Some(true);
        let body = self.build();

        let response = handler.execute(body).await?;

        #[allow(unused_mut)]
        let mut stream = AsyncStream::new(response);

        #[cfg(feature = "otel")]
        stream.aggregator.set_span(span);

        Ok(stream)
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
    #[cfg_attr(feature = "otel", instrument(
        name = "chat orpheus",
        fields(
            otel.kind = "client",
            otel.status_code = Empty,
            gen_ai.operation.name = "chat",
            gen_ai.system = "openrouter",
            gen_ai.output.type = Empty,
            gen_ai.request.choice.count = Empty,
            gen_ai.request.model = Empty,
            gen_ai.request.seed = Empty,
            gen_ai.request.frequency_penalty = Empty,
            gen_ai.request.max_tokens = Empty,
            gen_ai.request.presence_penalty = Empty,
            gen_ai.request.stop_sequences = Empty,
            gen_ai.request.temperature = Empty,
            gen_ai.request.top_k = Empty,
            gen_ai.request.top_p = Empty,
            gen_ai.response.finish_reasons = Empty,
            gen_ai.response.id = Empty,
            gen_ai.response.model = Empty,
            gen_ai.usage.input_tokens = Empty,
            gen_ai.usage.output_tokens = Empty,
        ),
        skip_all
    ))]
    pub fn chat(&self, messages: impl Into<ChatMessages>) -> ChatRequestBuilder {
        let handler = self.create_handler();
        ChatRequest::builder(
            #[cfg(feature = "otel")]
            Span::current(),
            Some(handler),
            messages,
        )
    }
}

#[derive(Debug)]
struct ChatHandler {
    builder: reqwest::RequestBuilder,
}

impl AsyncHandler for ChatHandler {
    const PATH: &str = CHAT_COMPLETION_PATH;
    type Input = ChatRequest;

    fn new(builder: reqwest::RequestBuilder) -> Self {
        ChatHandler { builder }
    }

    async fn execute(self, body: ChatRequest) -> Result<reqwest::Response> {
        #[cfg(feature = "otel")]
        {
            let span = &body.span;
            let _guard = span.enter();

            span.record(
                "gen_ai.output.type",
                if body.response_format.is_some() {
                    "json"
                } else {
                    "text"
                },
            );
            span.record(
                "gen_ai.request.model",
                body.model.as_deref().unwrap_or("default"),
            );
            if let Some(seed) = body.seed {
                span.record("gen_ai.request.seed", seed);
            }
            if let Some(frequency_penalty) = body.frequency_penalty {
                span.record("gen_ai.request.frequency_penalty", frequency_penalty);
            }
            if let Some(max_tokens) = body.max_tokens {
                span.record("gen_ai.request.max_tokens", max_tokens);
            }
            if let Some(presence_penalty) = body.presence_penalty {
                span.record("gen_ai.request.presence_penalty", presence_penalty);
            }
            if let Some(temperature) = body.temperature {
                span.record("gen_ai.request.temperature", temperature);
            }
            if let Some(top_k) = body.top_k {
                span.record("gen_ai.request.top_k", top_k);
            }
            if let Some(top_p) = body.top_p {
                span.record("gen_ai.request.top_p", top_p);
            }

            for message in body.messages.iter() {
                let content = message.content.to_string();
                match message.role {
                    Role::System | Role::Developer => {
                        info!(name: "gen_ai.system.message", content)
                    }
                    Role::User => {
                        info!(name: "gen_ai.user.message", content)
                    }
                    Role::Assistant => {
                        info!(name: "gen_ai.assistant.message", content)
                    }
                    Role::Tool => {
                        info!(name: "gen_ai.tool.message", content)
                    }
                }
            }
        }

        let response = self.builder.json(&body).send().await.map_err(Error::http)?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let err = response.text().await.map_err(Error::http)?;
            Err(Error::openrouter(err))
        }
    }
}
