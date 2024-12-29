pub use generation::Completion;
pub use payload::Prompt;
pub use stream::CompletionChunk;

mod tokens {
    use serde::Deserialize;

    #[pyo3::pyclass(get_all)]
    #[derive(Debug, Clone, Deserialize)]
    struct TopLogProbs {
        token: String,
        logprob: i32,
        bytes: Option<Vec<u8>>,
    }

    #[pyo3::pyclass(get_all)]
    #[derive(Debug, Clone, Deserialize)]
    struct Content {
        token: String,
        logprob: i32,
        bytes: Option<Vec<u8>>,
        top_logprobs: TopLogProbs,
    }

    #[pyo3::pyclass(get_all)]
    #[derive(Debug, Clone, Deserialize)]
    struct Refusal {
        token: String,
        logprob: i32,
        bytes: Option<Vec<u8>>,
        top_logprobs: TopLogProbs,
    }

    #[pyo3::pyclass(get_all)]
    #[derive(Debug, Clone, Deserialize)]
    pub struct LogProbs {
        content: Option<Vec<Content>>,
        refusal: Option<Vec<Refusal>>,
    }
}

mod message {
    use serde::{Deserialize, Serialize};
    use std::fmt;

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

    #[pyo3::pyclass]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    enum Part {
        Text { text: String },
        ImageUrl { image_url: ImageUrl },
        InputAudio { input_audio: InputAudio },
    }

    impl fmt::Display for Part {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Text { text } => write!(f, "Text(text={})", text),
                Self::ImageUrl { image_url } => write!(
                    f,
                    "Image(url={}, detail={})",
                    image_url.url,
                    image_url.detail.as_ref().unwrap_or(&"None".to_string())
                ),
                Self::InputAudio { input_audio } => write!(
                    f,
                    "Audio(data={}, format={})",
                    input_audio.data, input_audio.format
                ),
            }
        }
    }

    #[pyo3::pyclass]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged)]
    enum Content {
        Simple(String),
        Complex(Vec<Part>),
    }

    impl fmt::Display for Content {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Simple(content) => write!(f, "{}", content),
                Self::Complex(parts) => {
                    let mut buffer = String::new();
                    let mut iter = parts.iter().peekable();

                    buffer.push('[');
                    while let Some(part) = iter.next() {
                        buffer.push_str(part.to_string().as_str());

                        if iter.peek().is_some() {
                            buffer.push_str(", ")
                        }
                    }
                    buffer.push(']');

                    write!(f, "{}", buffer)
                }
            }
        }
    }

    #[pyo3::pymethods]
    impl Content {
        fn __repr__(&self) -> String {
            format!("{}", self)
        }
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
}

mod generation {
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
        system_fingerprint: String,
        object: String,
        usage: PromptUsage,
    }
}

mod stream {
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
}

mod payload {
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
}
