use super::message;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Prompt {
    messages: Vec<message::Message>,
    model: String,
    store: Option<Value>,
    metadata: Option<Value>,
    frequency_penalty: Option<i8>,
    logit_bias: Option<Value>,
    logprobs: Option<bool>,
    top_logprobs: Option<u8>,
    max_completion_tokens: Option<u32>,
    n: Option<u8>,
    modalities: Option<Vec<String>>,
    prediction: Option<Value>,
    audio: Option<Audio>,
    presence_penalty: Option<i8>,
    response_format: Option<Value>,
    seed: Option<u64>,
    service_tier: Option<String>,
    stop: Option<Stop>,
    stream: Option<bool>,
    stream_options: Option<StreamOptions>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    tools: Option<Vec<Tool>>,
    tool_choice: Option<ToolChoice>,
    parallel_tool_calls: Option<bool>,
    user: Option<String>,
}

impl Prompt {
    pub fn is_stream(&self) -> bool {
        self.stream.unwrap_or(false)
    }

    pub fn set_stream(&mut self, on: bool) {
        self.stream = Some(on)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Audio {
    voice: String,
    format: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Stop {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize)]
struct StreamOptions {
    include_usage: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Function {
    description: Option<String>,
    name: String,
    parameters: Option<Value>,
    strict: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Tool {
    r#type: String,
    function: Function,
}

#[derive(Debug, Serialize, Deserialize)]
struct FunctionOption {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolOption {
    r#type: String,
    function: FunctionOption,
}

#[derive(Debug, Serialize, Deserialize)]
enum ToolChoice {
    Mode(String),
    Select(ToolOption),
}

// Embeddings

#[derive(Debug, Serialize)]
enum EmbeddingInput {
    String(String),
    StringVector(Vec<String>),
    IntegerVector(Vec<f64>),
    NestedIntegerVector(Vec<Vec<f64>>),
}

#[derive(Debug, Serialize)]
struct EmbeddingPrompt {
    input: EmbeddingInput,
    model: String,
    encoding_format: Option<String>,
    dimensions: Option<i32>,
    user: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    index: i32,
    embedding: Vec<f64>,
    object: String,
}
