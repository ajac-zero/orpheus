use futures_lite::stream::Stream;
use pyo3::prelude::*;
use serde::Deserialize;
use std::pin::Pin;

use super::message;

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
struct PromptTokensDetails {
    audio_tokens: Option<u32>,
    cached_tokens: Option<u32>,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
struct CompletionTokensDetails {
    accepted_prediction_tokens: Option<u32>,
    audio_tokens: Option<u32>,
    reasoning_tokens: Option<u32>,
    rejected_prediction_tokens: Option<u32>,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
struct PromptUsage {
    completion_tokens: u32,
    prompt_tokens: u32,
    total_tokens: u32,
    completion_tokens_details: Option<CompletionTokensDetails>,
    prompt_tokens_details: Option<PromptTokensDetails>,
}

#[pyclass(frozen, get_all)]
#[derive(Debug, Deserialize)]
struct Choice {
    finish_reason: String,
    message: Py<message::Message>,
    index: u32,
    logprobs: Option<LogProbs>,
}

#[pyclass(frozen, get_all)]
#[derive(Debug, Deserialize)]
pub struct ChatCompletion {
    id: String,
    choices: Vec<Py<Choice>>,
    created: u64,
    model: String,
    service_tier: Option<String>,
    system_fingerprint: Option<String>,
    object: String,
    usage: PromptUsage,
}

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
    logprobs: Option<LogProbs>,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionChunk {
    id: String,
    choices: Vec<ChoiceChunk>,
    created: u64,
    model: String,
    service_tier: Option<String>,
    system_fingerprint: Option<String>,
    object: String,
    usage: Option<StreamUsage>,
}

#[pymethods]
impl ChatCompletionChunk {
    fn __repr__(&self) -> String {
        format!("ChatCompletionChunk(id={:?}, choices={:?}, created={:?}, model={:?}, service_tier={:?}, system_fingerprint={:?}, object={:?}, usage={:?})", self.id, self.choices, self.created, self.model, self.service_tier, self.system_fingerprint, self.object, self.usage)
    }
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
struct TopLogProbs {
    token: String,
    logprob: i32,
    bytes: Option<Vec<u8>>,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
struct Content {
    token: String,
    logprob: i32,
    bytes: Option<Vec<u8>>,
    top_logprobs: TopLogProbs,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
struct Refusal {
    token: String,
    logprob: i32,
    bytes: Option<Vec<u8>>,
    top_logprobs: TopLogProbs,
}

#[pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct LogProbs {
    content: Option<Vec<Content>>,
    refusal: Option<Vec<Refusal>>,
}

pub type BytesStream =
    Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + Sync>>;
