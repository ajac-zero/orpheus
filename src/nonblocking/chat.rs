#![allow(clippy::large_enum_variant)]

use async_std::sync::{Arc, Mutex};
use futures_lite::AsyncReadExt;
use isahc::AsyncReadResponseExt;
use pyo3::exceptions::{PyIOError, PyStopAsyncIteration, PyValueError};
use pyo3::prelude::*;
use pythonize::depythonize;

use crate::types::chat::{ChatCompletion, ChatCompletionChunk};
use crate::types::prompt::Prompt;
use crate::types::ExtrasMap;

use super::AsyncRest;

/// A non-blocking client for the chat completion API from OpenAI.
#[pyclass]
pub struct AsyncChat {
    client: Arc<isahc::HttpClient>,
    base_url: url::Url,
    api_key: String,
}

impl AsyncChat {
    pub fn new(client: Arc<isahc::HttpClient>, base_url: url::Url, api_key: String) -> Self {
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
    #[pyo3(signature = (extra_headers=None, extra_query=None, **py_kwargs))]
    async fn create(
        &self,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
        py_kwargs: Option<PyObject>,
    ) -> PyResult<CompletionResponse> {
        let args = py_kwargs.ok_or(PyValueError::new_err("No keyword arguments passed."))?;

        let prompt = Python::with_gil(|py| depythonize::<Prompt>(&args.into_bound(py)))
            .map_err(|e| PyValueError::new_err(format!("Invalid arguments: {}", e)))?;

        let mut response = self
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

        if prompt.is_stream() {
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
    body: Arc<Mutex<isahc::AsyncBody>>,
    lines: Arc<Mutex<Vec<String>>>,
}

impl Stream {
    fn new(response: isahc::Response<isahc::AsyncBody>) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(String::new())),
            body: Arc::new(Mutex::new(response.into_body())),
            lines: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[pymethods]
impl Stream {
    fn __anext__<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let lines = self.lines.clone();
        let body = self.body.clone();
        let buffer = self.buffer.clone();

        pyo3_async_runtimes::async_std::future_into_py(py, async move {
            let mut lines_ptr = lines.lock().await;
            let mut body_ptr = body.lock().await;
            let mut buffer_ptr = buffer.lock().await;

            let mut chunk_slice = [0; 1024];

            if let Some(line) = lines_ptr.pop() {
                if line == "data: [DONE]" {
                    Err(PyStopAsyncIteration::new_err("end of stream"))
                } else {
                    let data = &line[6..];

                    serde_json::from_str::<ChatCompletionChunk>(data)
                        .map_err(|e| PyValueError::new_err(format!("Failed to parse chunk: {}", e)))
                }
            } else {
                let chunk = body_ptr.read(&mut chunk_slice).await;

                match chunk {
                    Ok(0) => Err(PyStopAsyncIteration::new_err("end of stream")),
                    Ok(_) => {
                        let chunk_str = std::str::from_utf8(&chunk_slice)
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

                        Ok(ChatCompletionChunk::new_empty())
                    }
                    Err(e) => Err(PyValueError::new_err(format!(
                        "Failed to read chunk: {}",
                        e
                    ))),
                }
            }
        })
    }

    fn __aiter__(self_: PyRef<Self>) -> PyRef<Self> {
        self_
    }
}
