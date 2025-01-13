use super::{message, tokens};
use pyo3::prelude::*;
use serde::Deserialize;

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
struct StreamUsage {
    completion_tokens: u32,
    prompt_tokens: u32,
    total_tokens: u32,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
struct ChoiceChunk {
    finish_reason: Option<String>,
    delta: message::Delta,
    index: u32,
    logprobs: Option<tokens::LogProbs>,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct CompletionChunk {
    id: String,
    choices: Vec<ChoiceChunk>,
    created: u64,
    model: String,
    service_tier: Option<String>,
    system_fingerprint: Option<String>,
    object: String,
    usage: Option<StreamUsage>,
}
