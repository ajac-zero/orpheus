use super::{message, tokens};
use serde::Deserialize;

#[pyo3::pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
struct PromptTokensDetails {
    audio_tokens: Option<u32>,
    cached_tokens: Option<u32>,
}

#[pyo3::pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
struct CompletionTokensDetails {
    accepted_prediction_tokens: Option<u32>,
    audio_tokens: Option<u32>,
    reasoning_tokens: Option<u32>,
    rejected_prediction_tokens: Option<u32>,
}

#[pyo3::pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
struct PromptUsage {
    completion_tokens: u32,
    prompt_tokens: u32,
    total_tokens: u32,
    completion_tokens_details: Option<CompletionTokensDetails>,
    prompt_tokens_details: Option<PromptTokensDetails>,
}

#[pyo3::pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
struct Choice {
    finish_reason: String,
    message: message::Message,
    index: u32,
    logprobs: Option<tokens::LogProbs>,
}

#[pyo3::pyclass(get_all)]
#[derive(Debug, Deserialize)]
pub struct Completion {
    id: String,
    choices: Vec<Choice>,
    created: u64,
    model: String,
    service_tier: Option<String>,
    system_fingerprint: Option<String>,
    object: String,
    usage: PromptUsage,
}
