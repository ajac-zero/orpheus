use bon::Builder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPreferences {
    /// Sort preference (e.g., price, throughput).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningConfig {
    /// OpenAI-style reasoning effort setting
    pub effort: Option<ReasoningEffort>,

    /// Non-OpenAI-style reasoning effort setting. Cannot be used simultaneously with effort.
    pub max_tokens: Option<i32>,

    /// Whether to exclude reasoning from the response. Defaults to false
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_completion_request_serialization() {
        let request = CompletionRequest::builder()
            .model("gpt-3.5-turbo".into())
            .prompt("Hello, world!".into())
            .max_tokens(100)
            .build();

        let json = serde_json::to_string_pretty(&request).unwrap();
        println!("Serialized request:\n{}", json);

        // Test deserialization
        let deserialized: CompletionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.model, "gpt-3.5-turbo");
        assert_eq!(deserialized.prompt, "Hello, world!");
        assert_eq!(deserialized.max_tokens, Some(100));
    }
}
