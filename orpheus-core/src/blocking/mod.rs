pub mod chat;
mod common;
mod embed;

use std::env;

use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::prelude::*;
use reqwest::blocking::Client;
use serde_json::Value;

use crate::types::chat::message::Messages;
use crate::types::chat::prompt::ChatPrompt;
use crate::types::embed::{EmbeddingInput, EmbeddingPrompt, EmbeddingResponse};
use crate::types::ExtrasMap;
use crate::{API_KEY_ENVS, BASE_URL_ENVS};

use chat::{CompletionResponse, SyncChat};
use common::{Params, SyncRest};
use embed::SyncEmbed;

#[pyclass(frozen, subclass)]
pub struct OrpheusCore {
    params: Params,
}

impl SyncRest for OrpheusCore {
    fn get_params(&self) -> &Params {
        &self.params
    }
}

impl SyncChat for OrpheusCore {}
impl SyncEmbed for OrpheusCore {}

#[pymethods]
impl OrpheusCore {
    #[new]
    #[pyo3(signature = (base_url=None, api_key=None, default_headers=None, default_query=None))]
    fn new(
        base_url: Option<String>,
        api_key: Option<String>,
        default_headers: ExtrasMap,
        default_query: ExtrasMap,
    ) -> PyResult<Self> {
        let mut builder = Client::builder();

        if let Some(headers) = default_headers {
            let headermap = (&headers).try_into().expect("valid headers");
            builder = builder.default_headers(headermap);
        }

        let client = builder
            .user_agent("Orpheus")
            .use_rustls_tls()
            .build()
            .expect("should build http client");

        let mut base_url = base_url
            .or_else(|| env::var(BASE_URL_ENVS[0]).ok())
            .or_else(|| env::var(BASE_URL_ENVS[1]).ok())
            .and_then(|s| url::Url::parse(&s).ok())
            .ok_or(PyKeyError::new_err(format!(
                "{:?} environment variable not found.",
                BASE_URL_ENVS
            )))?;

        if let Some(params) = default_query {
            base_url.query_pairs_mut().extend_pairs(params);
        };

        let api_key = api_key
            .or_else(|| env::var(API_KEY_ENVS[0]).ok())
            .or_else(|| env::var(API_KEY_ENVS[1]).ok())
            .ok_or(PyKeyError::new_err(format!(
                "{:?} environment variable not found.",
                API_KEY_ENVS
            )))?;

        Ok(Self {
            params: Params::new(client, base_url, api_key),
        })
    }

    #[pyo3(signature = (input, model, dimensions=None, encoding_format=None, user=None, extra_headers=None, extra_query=None))]
    fn native_embeddings_create(
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

        let completion = self
            .embeddings(prompt, extra_headers, extra_query)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(completion)
    }

    #[pyo3(signature = (model, messages, stream=None, extra_headers=None, extra_query=None, extra=None))]
    fn native_chat_completions_create(
        &self,
        model: String,
        messages: Messages,
        stream: Option<bool>,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
        extra: Option<&[u8]>,
    ) -> PyResult<CompletionResponse> {
        let extra = extra
            .map(serde_json::from_slice::<Value>)
            .transpose()
            .expect("Serialize bytes to json");

        let prompt = ChatPrompt::new(model, &messages, stream, extra);

        let completion = self
            .chat_completion(&prompt, extra_headers, extra_query)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(completion)
    }
}
