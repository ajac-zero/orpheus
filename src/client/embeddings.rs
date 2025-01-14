#![allow(clippy::too_many_arguments)]

use super::{build_request, ExtrasMap};
use isahc::{AsyncReadResponseExt, HttpClient, ReadResponseExt};
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;

#[derive(Debug, Serialize, FromPyObject)]
enum EmbeddingInput {
    String(String),
    StringVector(Vec<String>),
    IntegerVector(Vec<f64>),
    NestedIntegerVector(Vec<Vec<f64>>),
}

#[derive(Debug, Serialize)]
pub struct EmbeddingPrompt {
    input: EmbeddingInput,
    model: String,
    encoding_format: Option<String>,
    dimensions: Option<i32>,
    user: Option<String>,
}

#[derive(Debug, Deserialize, IntoPyObject)]
pub struct EmbeddingResponse {
    index: i32,
    embedding: Vec<f64>,
    object: String,
}

#[pyclass]
pub struct SyncEmbeddings {
    client: Arc<HttpClient>,
    url: Url,
    api_key: String,
}

impl SyncEmbeddings {
    pub fn new(client: Arc<HttpClient>, url: Url, api_key: String) -> Self {
        Self {
            client,
            url,
            api_key,
        }
    }
}

#[pymethods]
impl SyncEmbeddings {
    #[pyo3(signature = (input, model, dimensions=None, encoding_format=None, user=None, extra_headers=None, extra_query=None))]
    fn create(
        &self,
        input: EmbeddingInput,
        model: String,
        dimensions: Option<i32>,
        encoding_format: Option<String>,
        user: Option<String>,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
    ) -> PyResult<EmbeddingResponse> {
        let prompt = EmbeddingPrompt {
            input,
            model,
            dimensions,
            encoding_format,
            user,
        };

        let request = build_request(
            "/v1/embeddings",
            &prompt,
            &self.url,
            &self.api_key,
            extra_headers,
            extra_query,
        );

        let mut response = self.client.send(request).expect("Receive response");

        match response.status().into() {
            200 => {
                let completion = response.json::<EmbeddingResponse>().map_err(|e| {
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

#[pyclass]
pub struct AsyncEmbeddings {
    client: Arc<HttpClient>,
    url: Url,
    api_key: String,
}

#[pymethods]
impl AsyncEmbeddings {
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
        let prompt = EmbeddingPrompt {
            input,
            model,
            dimensions,
            encoding_format,
            user,
        };

        let request = build_request(
            "/v1/embeddings",
            &prompt,
            &self.url,
            &self.api_key,
            extra_headers,
            extra_query,
        );

        let mut response = self
            .client
            .send_async(request)
            .await
            .expect("Receive response");

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
