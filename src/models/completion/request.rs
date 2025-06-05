use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// The model ID to use. If unspecified, the user's default is used.
    pub model: String,

    /// The text prompt to complete
    pub prompt: String,

    /// Alternate list of models for routing overrides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,

    /// Preferences for provider routing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderPreferences>,

    /// Configuration for model reasoning/thinking tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConfig>,

    /// Whether to include usage information in the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<UsageConfig>,

    /// List of prompt transforms (OpenRouter-only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transforms: Option<Vec<String>>,

    /// Enable streaming of results. Defaults to false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Maximum number of tokens (range: [1, context_length)).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,

    /// Sampling temperature (range: [0, 2]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    /// Seed for deterministic outputs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,

    /// Top-p sampling value (range: (0, 1]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,

    /// Top-k sampling value (range: [1, Infinity)).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,

    /// Frequency penalty (range: [-2, 2]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,

    /// Presence penalty (range: [-2, 2]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,

    /// Repetition penalty (range: (0, 2]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repetition_penalty: Option<f64>,

    /// Mapping of token IDs to bias values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, f64>>,

    /// Number of top log probabilities to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<i32>,

    /// Minimum probability threshold (range: [0, 1]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_p: Option<f64>,

    /// Alternate top sampling parameter (range: [0, 1]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_a: Option<f64>,

    /// A stable identifier for your end-users. Used to help detect and prevent abuse.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPreferences {
    /// Sort preference (e.g., price, throughput).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningConfig {
    /// OpenAI-style reasoning effort setting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<ReasoningEffort>,

    /// Non-OpenAI-style reasoning effort setting. Cannot be used simultaneously with effort.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,

    /// Whether to exclude reasoning from the response. Defaults to false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageConfig {
    /// Whether to include usage information in the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<bool>,
}
