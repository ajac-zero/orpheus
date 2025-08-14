use std::collections::HashMap;

use bon::{Builder, builder};
use serde::Serialize;

use crate::{
    Error, Result,
    models::{
        common::{
            handler::{AsyncExecutor, Executor},
            mode::{Async, Mode, Sync},
            provider::ProviderPreferences,
            reasoning::ReasoningConfig,
            usage::UsageConfig,
        },
        completion::{CompletionHandler, CompletionResponse},
    },
};
use completion_request_builder::{IsComplete, State};

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct CompletionRequest<M: Mode> {
    #[serde(skip)]
    #[builder(start_fn)]
    handler: Option<CompletionHandler<M>>,

    /// The text prompt to complete
    #[builder(start_fn)]
    pub prompt: String,

    /// The model ID to use. If unspecified, the user's default is used.
    pub model: String,

    /// Alternate list of models for routing overrides.
    pub models: Option<Vec<String>>,

    /// Preferences for provider routing.
    pub provider: Option<ProviderPreferences>,

    /// Configuration for model reasoning/thinking tokens
    pub reasoning: Option<ReasoningConfig>,

    /// Whether to include usage information in the response
    pub usage: Option<UsageConfig>,

    /// List of prompt transforms (OpenRouter-only).
    pub transforms: Option<Vec<String>>,

    /// Enable streaming of results. Defaults to false
    pub stream: Option<bool>,

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

impl<S: State> CompletionRequestBuilder<Sync, S>
where
    S: IsComplete,
{
    pub fn send(mut self) -> Result<CompletionResponse> {
        let handler = self.handler.take().expect("Has handler");

        let body = self.build();

        let response = handler.execute(body)?;

        let completion_response = response.json().map_err(Error::http)?;

        Ok(completion_response)
    }
}

impl<S: State> CompletionRequestBuilder<Async, S>
where
    S: IsComplete,
{
    pub async fn send(mut self) -> Result<CompletionResponse> {
        let handler = self.handler.take().expect("Has handler");

        let body = self.build();

        let response = handler.execute(body).await?;

        let completion_response = response.json().await.map_err(Error::http)?;

        Ok(completion_response)
    }
}
