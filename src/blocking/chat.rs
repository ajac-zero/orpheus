use std::collections::VecDeque;
use std::io::Read;
use std::sync::Arc;

use pyo3::exceptions::{PyIOError, PyStopIteration, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pythonize::depythonize;
use reqwest::blocking::{Client, Response};

use crate::types::chat::{ChatCompletion, ChatCompletionChunk};
use crate::types::prompt::Prompt;
use crate::types::ExtrasMap;

use super::SyncRest;

/// A blocking client for the chat completion API from OpenAI.
#[pyclass]
pub struct SyncChat {
    client: Arc<Client>,
    base_url: url::Url,
    api_key: String,
}

impl SyncChat {
    pub fn new(client: Arc<Client>, base_url: url::Url, api_key: String) -> Self {
        Self {
            client,
            base_url,
            api_key,
        }
    }
}

// Compose traits to send REST requests.
impl SyncRest for SyncChat {}

#[pymethods]
impl SyncChat {
    #[pyo3(signature = (extra_headers=None, extra_query=None, **py_kwargs))]
    fn create(
        &self,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
        py_kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<CompletionResponse> {
        let args = py_kwargs.ok_or(PyValueError::new_err("No keyword arguments passed."))?;

        let prompt = depythonize::<Prompt>(args)
            .map_err(|e| PyValueError::new_err(format!("Invalid arguments: {}", e)))?;

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
            .map_err(|e| PyIOError::new_err(format!("Failed to send request: {}", e)))?;

        if response.status() == 401 {
            return Err(PyIOError::new_err(
                "401 (Unauthorized) response; Is the API key valid?",
            ));
        };

        if prompt.is_stream() {
            let stream = Stream::new(response);

            Ok(CompletionResponse::Stream(stream))
        } else {
            let completion = response
                .json::<ChatCompletion>()
                .map_err(|e| PyValueError::new_err(format!("Failed to parse response: {}", e)))?;

            Ok(CompletionResponse::Completion(completion))
        }
    }

    #[getter]
    fn completions(self_: PyRef<Self>) -> PyRef<Self> {
        self_
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(pyo3::IntoPyObject)]
enum CompletionResponse {
    #[pyo3(transparent)]
    Completion(ChatCompletion),
    #[pyo3(transparent)]
    Stream(Stream),
}

#[pyclass]
struct Stream {
    buffer: String,
    body: Response,
    chunk: [u8; 1024],
    lines: VecDeque<String>,
}

impl Stream {
    fn new(response: Response) -> Self {
        Self {
            buffer: String::new(),
            body: response,
            chunk: [0; 1024],
            lines: VecDeque::new(),
        }
    }
}

#[pymethods]
impl Stream {
    fn __next__(&mut self) -> PyResult<ChatCompletionChunk> {
        if let Some(line) = self.lines.pop_front() {
            if line == "data: [DONE]" {
                Err(PyStopIteration::new_err("end of stream"))
            } else {
                let data = &line[6..];

                serde_json::from_str::<ChatCompletionChunk>(data)
                    .map_err(|e| PyValueError::new_err(format!("Failed to parse chunk: {}", e)))
            }
        } else {
            let chunk = self.body.read(&mut self.chunk);

            match chunk {
                Ok(0) => Err(PyStopIteration::new_err("end of stream")),
                Ok(_) => {
                    let chunk_str = std::str::from_utf8(&self.chunk)
                        .expect("should convert chunk to string")
                        .trim_end_matches('\0');
                    self.buffer.push_str(chunk_str);

                    if self.buffer.ends_with("\n\n") {
                        self.lines = self
                            .buffer
                            .lines()
                            .filter(|l| !l.is_empty())
                            .map(|l| l.to_string())
                            .collect::<VecDeque<String>>();

                        self.buffer.clear();
                    };

                    self.__next__()
                }
                Err(e) => Err(PyValueError::new_err(format!(
                    "Failed to read chunk: {}",
                    e
                ))),
            }
        }
    }

    fn __iter__(self_: PyRef<Self>) -> PyRef<Self> {
        self_
    }
}
