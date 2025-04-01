use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use super::message;

#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct ChatPrompt<'a> {
    messages: &'a message::Messages,
    model: String,
    stream: Option<bool>,
    tools: Option<Vec<Tool>>,
    tool_choice: Option<ToolChoice>,
    #[serde(flatten)]
    extra: Option<Value>,
}

impl<'a> ChatPrompt<'a> {
    pub fn new(
        model: String,
        messages: &'a message::Messages,
        stream: Option<bool>,
        extra: Option<Value>,
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

    pub fn is_stream(&self) -> bool {
        self.stream.is_some_and(|x| x)
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
