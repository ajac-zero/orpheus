#![allow(clippy::large_enum_variant)]

use std::io::{BufRead, BufReader, Lines};

use pyo3::{
    exceptions::{PyIOError, PyStopIteration, PyValueError},
    prelude::*,
    types::PyDict,
};
use pythonize::depythonize;
use reqwest::blocking::{Client, Response};
use serde_json::Value;

use crate::types::{
    chat::{ChatCompletion, ChatCompletionChunk},
    message::{EitherMessages, Messages},
    prompt::Prompt,
    ExtrasMap,
};

use super::SyncRest;

/// A blocking client for the chat completion API from OpenAI.
#[pyclass]
pub struct SyncChat {
    client: Client,
    base_url: url::Url,
    api_key: String,
}

impl SyncChat {
    pub fn new(client: Client, base_url: url::Url, api_key: String) -> Self {
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
    #[pyo3(signature = (model, messages, stream=None, extra_headers=None, extra_query=None, **extra))]
    fn create(
        &self,
        py: Python,
        model: String,
        messages: EitherMessages,
        stream: Option<bool>,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
        extra: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<CompletionResponse> {
        let messages = messages.map_left(Ok).left_or_else(|x| {
            x.extract::<Messages>(py)
                .map(|x| Py::new(py, x).expect("bind to GIL"))
        })?;

        let extra = extra.map(|x| depythonize::<Value>(x)).transpose()?;

        let prompt = Prompt::new(model, messages.get(), stream, extra);

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

        if stream.is_some_and(|x| x) {
            let buffer = BufReader::new(response).lines();
            let stream = Stream::new(buffer);

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

#[derive(IntoPyObject)]
enum CompletionResponse {
    #[pyo3(transparent)]
    Completion(ChatCompletion),
    #[pyo3(transparent)]
    Stream(Stream),
}

#[pyclass]
struct Stream {
    buffer: Lines<BufReader<Response>>,
}

impl Stream {
    fn new(buffer: Lines<BufReader<Response>>) -> Self {
        Self { buffer }
    }
}

#[pymethods]
impl Stream {
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
