use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::message;

pub type Kwargs = HashMap<String, Value>;

#[derive(Debug, Serialize)]
pub struct Prompt<'a> {
    messages: &'a message::Messages,
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<ToolChoice>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    extra: Option<Kwargs>,
}

impl<'a> Prompt<'a> {
    pub fn new(
        model: String,
        messages: &'a message::Messages,
        stream: Option<bool>,
        extra: Option<Kwargs>,
    ) -> Self {
        Self {
            model,
            messages,
            stream,
            tools: None,
            tool_choice: None,
            extra,
        }
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
pub struct EmbeddingPrompt {
    input: EmbeddingInput,
    model: String,
    encoding_format: Option<String>,
    dimensions: Option<i32>,
    user: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingResponse {
    index: i32,
    embedding: Vec<f64>,
    object: String,
}
