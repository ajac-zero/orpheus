#![allow(clippy::large_enum_variant)]

use std::sync::Arc;

use futures_lite::stream::StreamExt;
use pyo3::exceptions::{PyIOError, PyStopAsyncIteration, PyValueError};
use pyo3::prelude::*;
use pythonize::depythonize;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::types::chat::message::{EitherMessages, Message};
use crate::types::chat::prompt::ChatPrompt;
use crate::types::chat::{BytesStream, ChatCompletion, ChatCompletionChunk};
use crate::types::ExtrasMap;

use super::AsyncRest;

/// A non-blocking client for the chat completion API from OpenAI.
#[pyclass(frozen)]
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
    #[pyo3(signature = (model, messages, stream=None, extra_headers=None, extra_query=None, **py_kwargs))]
    async fn create(
        &self,
        model: String,
        messages: EitherMessages,
        stream: Option<bool>,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
        py_kwargs: Option<PyObject>,
    ) -> PyResult<CompletionResponse> {
        let messages = messages.map_left(Ok).left_or_else(|x| {
            Python::with_gil(|py| {
                x.into_bound(py)
                    .into_iter()
                    .map(|x| {
                        x.extract::<Message>()
                            .map(|x| Py::new(py, x).expect("bind to GIL"))
                    })
                    .collect()
            })
        })?;

        let extra = py_kwargs
            .map(|x| Python::with_gil(|py| depythonize::<Value>(&x.into_bound(py))))
            .transpose()?;

        let prompt = ChatPrompt::new(model, &messages, stream, extra);

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

        if stream.is_some_and(|x| x) {
            let bytes_steam = Box::pin(response.bytes_stream());
            let stream = StreamCompletion::new(bytes_steam);

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
    Stream(StreamCompletion),
}

#[pyclass]
struct StreamCompletion {
    bytes_stream: Arc<Mutex<BytesStream>>,
}

impl StreamCompletion {
    fn new(bytes_stream: BytesStream) -> Self {
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
impl StreamCompletion {
    fn __anext__<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let bytes_stream = Arc::clone(&self.bytes_stream);

        pyo3_async_runtimes::tokio::future_into_py(py, async {
            pyo3_async_runtimes::tokio::get_runtime()
                .spawn(StreamCompletion::next(bytes_stream))
                .await
                .expect("h")
        })
    }

    fn __aiter__(self_: PyRef<Self>) -> PyRef<Self> {
        self_
    }
}
