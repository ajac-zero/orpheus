#![allow(clippy::too_many_arguments)]

use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;

use crate::types::embed::{EmbeddingInput, EmbeddingPrompt, EmbeddingResponse};
use crate::types::ExtrasMap;

use super::AsyncRest;

/// A nonblocking client for the chat completion API from OpenAI.
#[pyclass]
pub struct AsyncEmbed {
    client: reqwest::Client,
    base_url: url::Url,
    api_key: String,
}

impl AsyncEmbed {
    pub fn new(client: reqwest::Client, base_url: url::Url, api_key: String) -> Self {
        Self {
            client,
            base_url,
            api_key,
        }
    }
}

// Compose traits to send REST requests.
impl AsyncRest for AsyncEmbed {}

#[pymethods]
impl AsyncEmbed {
    #[pyo3(signature = (input, model, dimensions=None, encoding_format=None, user=None, extra_headers=None, extra_query=None))]
    async fn create(
        &self,
        input: EmbeddingInput,
        model: String,
        dimensions: Option<i32>,
        encoding_format: Option<String>,
        user: Option<String>,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
    ) -> PyResult<EmbeddingResponse> {
        let prompt = EmbeddingPrompt::new(input, model, encoding_format, dimensions, user);

        let response = self
            .api_request(
                &self.client,
                &self.base_url,
                &self.api_key,
                "/v1/embeddings",
                &prompt,
                extra_headers,
                extra_query,
            )
            .await
            .map_err(|e| PyIOError::new_err(format!("Failed to send request: {}", e)))?;

        match response.status().into() {
            200 => {
                let completion = response.json::<EmbeddingResponse>().await.map_err(|e| {
                    PyValueError::new_err(format!("Failed to parse response: {}", e))
                })?;

                Ok(completion)
            }
            401 => Err(PyIOError::new_err(
                "401 (Unauthorized) response; Is the API key valid?",
            )),
            code => Err(PyIOError::new_err(format!(
                "Unexpected status code: {}",
                code
            ))),
        }
    }
}
