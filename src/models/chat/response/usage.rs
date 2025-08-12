use serde::{Deserialize, Serialize};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatUsage {
    /// Number of tokens in the prompt
    pub prompt_tokens: u32,

    /// Number of tokens in the completion
    pub completion_tokens: u32,

    /// Total number of tokens used
    pub total_tokens: u32,

    /// Detailed prompt token information
    pub prompt_tokens_details: Option<PromptTokensDetails>,

    /// Detailed completion token information
    pub completion_tokens_details: Option<CompletionTokensDetails>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTokensDetails {
    /// Number of cached tokens in the prompt
    pub cached_tokens: u32,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionTokensDetails {
    /// Number of reasoning tokens in the completion
    pub reasoning_tokens: u32,
}
