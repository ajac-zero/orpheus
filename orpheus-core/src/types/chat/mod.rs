pub mod message;
pub mod prompt;

use futures_lite::stream::{Stream, StreamExt};
use pyo3::exceptions::{PyStopAsyncIteration, PyStopIteration, PyValueError};
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::{future_into_py, get_runtime};
use reqwest::blocking::Response;
use serde::Deserialize;
use std::io::{BufReader, Lines};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

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

#[pyclass]
pub struct ChunkStream {
    buffer: Lines<BufReader<Response>>,
}

impl ChunkStream {
    pub fn new(buffer: Lines<BufReader<Response>>) -> Self {
        Self { buffer }
    }
}

#[pymethods]
impl ChunkStream {
    fn __next__(&mut self) -> PyResult<ChatCompletionChunk> {
        loop {
            match self.buffer.next() {
                Some(chunk) => {
                    let line = chunk.unwrap();

                    if line.is_empty() {
                        continue;
                    }

                    let data = &line[6..];

                    if data == "[DONE]" {
                        break Err(PyStopIteration::new_err("end of stream"));
                    }

                    break serde_json::from_str::<ChatCompletionChunk>(data)
                        .map_err(|e| PyValueError::new_err(format!("{e}")));
                }
                None => break Err(PyStopIteration::new_err("end of stream")),
            }
        }
    }

    fn __iter__(self_: PyRef<Self>) -> PyRef<Self> {
        self_
    }
}

#[pyclass(frozen)]
pub struct AsyncChunkStream {
    bytes_stream: Arc<Mutex<BytesStream>>,
}

impl AsyncChunkStream {
    pub fn new(bytes_stream: BytesStream) -> Self {
        Self {
            bytes_stream: Arc::new(Mutex::new(bytes_stream)),
        }
    }

    async fn next(bytes_stream: Arc<Mutex<BytesStream>>) -> PyResult<ChatCompletionChunk> {
        let mut stream_guard = bytes_stream.lock().await;

        if let Some(Ok(bytes)) = stream_guard.next().await {
            let line = String::from_utf8(bytes.to_vec())
                .map_err(|x| PyValueError::new_err(x.to_string()))?;

            let data = &line[6..];

            if data == "[DONE]\n\n" {
                return Err(PyStopAsyncIteration::new_err("end of stream"));
            }

            serde_json::from_str::<ChatCompletionChunk>(data)
                .map_err(|x| PyValueError::new_err(x.to_string()))
        } else {
            Err(PyStopAsyncIteration::new_err("end of stream"))
        }
    }
}

#[pymethods]
impl AsyncChunkStream {
    fn __anext__<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let bytes_stream = Arc::clone(&self.bytes_stream);

        future_into_py(py, async {
            get_runtime()
                .spawn(AsyncChunkStream::next(bytes_stream))
                .await
                .expect("h")
        })
    }

    fn __aiter__(self_: PyRef<Self>) -> PyRef<Self> {
        self_
    }
}
