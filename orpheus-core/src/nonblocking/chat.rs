use either::Either;
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use pythonize::depythonize;
use serde_json::Value;

use crate::types::chat::message::Messages;
use crate::types::chat::prompt::ChatPrompt;
use crate::types::chat::{AsyncChunkStream, ChatCompletion};
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

type CompletionResponse = Either<ChatCompletion, AsyncChunkStream>;

#[pymethods]
impl AsyncChat {
    #[pyo3(signature = (model, messages, stream=None, extra_headers=None, extra_query=None, **py_kwargs))]
    async fn create(
        &self,
        model: String,
        messages: Messages,
        stream: Option<bool>,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
        py_kwargs: Option<PyObject>,
    ) -> PyResult<CompletionResponse> {
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

        let completion = if prompt.is_stream() {
            let bytes_steam = Box::pin(response.bytes_stream());
            let stream = AsyncChunkStream::new(bytes_steam);

            Either::Right(stream)
        } else {
            let completion = response
                .json::<ChatCompletion>()
                .await
                .map_err(|e| PyValueError::new_err(format!("Failed to parse response: {}", e)))?;

            Either::Left(completion)
        };

        Ok(completion)
    }
}
