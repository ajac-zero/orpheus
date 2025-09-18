use std::collections::HashMap;

use bon::{Builder, builder};
use completion_request_builder::{IsComplete, State};
use serde::Serialize;

use crate::{
    Result,
    client::{
        core::Pool,
        mode::{Async, Mode, Sync},
    },
    constants::COMPLETION_PATH,
    models::{Preferences, Reasoning, Usage, completion::CompletionResponse},
};

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into),
    builder_type(vis = "pub", doc {
        /// Builder to set the parameters of a completion request
    })
)]
pub(crate) struct CompletionRequest<'a, M: Mode> {
    #[serde(skip)]
    #[builder(start_fn)]
    pool: &'a Pool<M>,

    #[serde(skip)]
    #[builder(start_fn)]
    api_key: Option<&'a str>,

    /// The text prompt to complete
    #[builder(start_fn)]
    pub prompt: String,

    /// The model ID to use. If unspecified, the user's default is used.
    pub model: String,

    /// Alternate list of models for routing overrides.
    pub models: Option<Vec<String>>,

    /// Preferences for provider routing.
    pub provider: Option<Preferences>,

    /// Configuration for model reasoning/thinking tokens
    pub reasoning: Option<Reasoning>,

    /// Whether to include usage information in the response
    pub usage: Option<Usage>,

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

impl<'a, S: State> CompletionRequestBuilder<'a, Sync, S>
where
    S: IsComplete,
{
    pub fn send(self) -> Result<CompletionResponse> {
        let mut handler = self.pool.get().expect("Has handler");
        let token = self.api_key;
        let body = self.build();

        let response = handler
            .execute()
            .segments(&[COMPLETION_PATH])
            .payload(body)
            .maybe_token(token)
            .call()?;

        let completion_response = response.json()?;

        Ok(completion_response)
    }
}

impl<'a, S: State> CompletionRequestBuilder<'a, Async, S>
where
    S: IsComplete,
{
    pub async fn send(self) -> Result<CompletionResponse> {
        let mut handler = self.pool.get().await.expect("Has handler");
        let token = self.api_key;
        let body = self.build();

        let response = handler
            .execute()
            .segments(&[COMPLETION_PATH])
            .payload(body)
            .maybe_token(token)
            .call()
            .await?;

        let completion_response = response.json().await?;

        Ok(completion_response)
    }
}
