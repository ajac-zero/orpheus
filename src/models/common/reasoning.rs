use serde::{Deserialize, Serialize};

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
