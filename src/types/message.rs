use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Function {
    name: String,
    arguments: String,
}

#[pyo3::pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    id: String,
    r#type: String,
    function: Function,
}

#[pyo3::pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Audio {
    id: String,
    expires_at: u64,
    data: String,
    transcript: String,
}

#[pyo3::pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ImageUrl {
    url: String,
    detail: Option<String>,
}

#[pyo3::pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct InputAudio {
    data: String,
    format: String,
}

#[derive(pyo3::IntoPyObject, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Part {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
    InputAudio { input_audio: InputAudio },
}

#[derive(pyo3::IntoPyObject, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum Content {
    Simple(String),
    Complex(Vec<Part>),
}

#[pyo3::pyclass(get_all)]
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    role: String,
    content: Option<Content>,
    refusal: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
    audio: Option<Audio>,
}

#[pyo3::pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct Delta {
    role: Option<String>,
    content: Option<Content>,
    refusal: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}
