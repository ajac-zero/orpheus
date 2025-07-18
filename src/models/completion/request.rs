use std::collections::HashMap;

use serde::Serialize;

use crate::models::common::{
    provider::ProviderPreferences, reasoning::ReasoningConfig, usage::UsageConfig,
};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
pub struct CompletionRequest {
    /// The model ID to use. If unspecified, the user's default is used.
    pub model: String,

    /// The text prompt to complete
    pub prompt: String,

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

impl CompletionRequest {
    pub fn new(
        model: String,
        prompt: String,
        models: Option<Vec<String>>,
        provider: Option<ProviderPreferences>,
        reasoning: Option<ReasoningConfig>,
        usage: Option<UsageConfig>,
        transforms: Option<Vec<String>>,
        stream: Option<bool>,
        max_tokens: Option<i32>,
        temperature: Option<f64>,
        seed: Option<i32>,
        top_p: Option<f64>,
        top_k: Option<i32>,
        frequency_penalty: Option<f64>,
        presence_penalty: Option<f64>,
        repetition_penalty: Option<f64>,
        logit_bias: Option<HashMap<String, f64>>,
        top_logprobs: Option<i32>,
        min_p: Option<f64>,
        top_a: Option<f64>,
        user: Option<String>,
    ) -> Self {
        Self {
            model,
            prompt,
            models,
            provider,
            reasoning,
            usage,
            transforms,
            stream,
            max_tokens,
            temperature,
            seed,
            top_p,
            top_k,
            frequency_penalty,
            presence_penalty,
            repetition_penalty,
            logit_bias,
            top_logprobs,
            min_p,
            top_a,
            user,
        }
    }
}
