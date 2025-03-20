#![allow(clippy::large_enum_variant)]

use std::collections::VecDeque;
use std::sync::Arc;

use pyo3::exceptions::{PyIOError, PyStopAsyncIteration, PyValueError};
use pyo3::prelude::*;
use pythonize::depythonize;
use tokio::sync::Mutex;

use crate::types::chat::{ChatCompletion, ChatCompletionChunk};
use crate::types::message::{EitherMessages, Messages};
use crate::types::prompt::{Kwargs, Prompt};
use crate::types::ExtrasMap;

use super::AsyncRest;

/// A non-blocking client for the chat completion API from OpenAI.
#[pyclass]
pub struct AsyncChat {
    client: reqwest::Client,
    base_url: url::Url,
    api_key: String,
}

impl AsyncChat {
    pub fn new(client: reqwest::Client, base_url: url::Url, api_key: String) -> Self {
        Self {
            client,
            base_url,
            api_key,
        }
    }
}

// Compose traits to send non-blocking REST requests.
impl AsyncRest for AsyncChat {}

#[pymethods]
impl AsyncChat {
    #[pyo3(signature = (model, messages, stream=false, extra_headers=None, extra_query=None, **py_kwargs))]
    async fn create(
        &self,
        model: String,
        messages: EitherMessages,
        stream: bool,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
        py_kwargs: Option<PyObject>,
    ) -> PyResult<CompletionResponse> {
        let messages = messages
            .map_left(Ok)
            .left_or_else(|x| Python::with_gil(|py| x.extract::<Py<Messages>>(py)))?;

        let extra = py_kwargs
            .map(|x| Python::with_gil(|py| depythonize::<Kwargs>(&x.into_bound(py))))
            .transpose()?;

        let prompt = Prompt::new(model, messages.get(), extra);

        let response = self
            .api_request(
                &self.client,
                &self.base_url,
                &self.api_key,
                "/chat/completions",
                &prompt,
                extra_headers,
                extra_query,
            )
            .await
            .map_err(|e| PyIOError::new_err(format!("Failed to send request: {}", e)))?;

        if stream {
            let stream = Stream::new(response);

            Ok(CompletionResponse::Stream(stream))
        } else {
            let completion = response
                .json::<ChatCompletion>()
                .await
                .map_err(|e| PyValueError::new_err(format!("Failed to parse response: {}", e)))?;

            Ok(CompletionResponse::Completion(completion))
        }
    }

    #[getter]
    fn completions(self_: PyRef<Self>) -> PyRef<Self> {
        self_
    }
}

#[derive(pyo3::IntoPyObject)]
enum CompletionResponse {
    #[pyo3(transparent)]
    Completion(ChatCompletion),
    #[pyo3(transparent)]
    Stream(Stream),
}

#[pyclass]
struct Stream {
    buffer: Arc<Mutex<String>>,
    body: Arc<Mutex<reqwest::Response>>,
    lines: Arc<Mutex<VecDeque<String>>>,
}

impl Stream {
    fn new(response: reqwest::Response) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(String::new())),
            body: Arc::new(Mutex::new(response)),
            lines: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

async fn next(
    buffer: Arc<Mutex<String>>,
    body: Arc<Mutex<reqwest::Response>>,
    lines: Arc<Mutex<VecDeque<String>>>,
) -> PyResult<ChatCompletionChunk> {
    let mut lines_ptr = lines.lock().await;
    let mut body_ptr = body.lock().await;
    let mut buffer_ptr = buffer.lock().await;

    loop {
        if let Some(line) = lines_ptr.pop_front() {
            if line == "data: [DONE]" {
                break Err(PyStopAsyncIteration::new_err("end of stream"));
            } else {
                let data = &line[6..];

                break serde_json::from_str::<ChatCompletionChunk>(data)
                    .map_err(|e| PyValueError::new_err(format!("Failed to parse chunk: {}", e)));
            }
        } else {
            let chunk = body_ptr.chunk().await.expect("chunk methof");

            match chunk {
                Some(bytes) => {
                    let chunk_str = std::str::from_utf8(&bytes)
                        .expect("should convert chunk to string")
                        .trim_end_matches('\0');
                    buffer_ptr.push_str(chunk_str);

                    if let Some(position) = buffer_ptr.find("\n\n") {
                        let split_point = position + 2;
                        let moved = buffer_ptr.drain(..split_point).collect::<String>();
                        let new_lines = moved
                            .lines()
                            .filter(|l| !l.is_empty())
                            .map(|l| l.to_string());

                        lines_ptr.extend(new_lines);
                    };

                    continue;
                }
                None => break Err(PyStopAsyncIteration::new_err("end of stream")),
            }
        }
    }
}

#[pymethods]
impl Stream {
    fn __anext__<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let lines = self.lines.clone();
        let body = self.body.clone();
        let buffer = self.buffer.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async {
            pyo3_async_runtimes::tokio::get_runtime()
                .spawn(next(buffer, body, lines))
                .await
                .expect("h")
        })
    }

    fn __aiter__(self_: PyRef<Self>) -> PyRef<Self> {
        self_
    }
}
