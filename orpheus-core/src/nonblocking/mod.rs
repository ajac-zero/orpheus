mod chat;
mod common;
mod embed;

use std::env;

use chat::{AsyncChat, CompletionResponse};
use common::{AsyncRest, Params};
use embed::AsyncEmbed;
use pyo3::{
    exceptions::{PyKeyError, PyValueError},
    prelude::*,
};
use serde_json::Value;

use crate::{
    API_KEY_ENVS, BASE_URL_ENVS,
    types::{
        ExtrasMap,
        chat::{message::Messages, prompt::ChatPrompt},
        embed::{EmbeddingInput, EmbeddingPrompt, EmbeddingResponse},
    },
};

#[pyclass(frozen, subclass)]
pub struct AsyncOrpheusCore {
    params: Params,
}

impl AsyncRest for AsyncOrpheusCore {
    fn get_params(&self) -> &Params {
        &self.params
    }
}
impl AsyncChat for AsyncOrpheusCore {}
impl AsyncEmbed for AsyncOrpheusCore {}

#[pymethods]
impl AsyncOrpheusCore {
    #[new]
    #[pyo3(signature = (base_url=None, api_key=None, default_headers=None, default_query=None))]
    fn new(
        base_url: Option<String>,
        api_key: Option<String>,
        default_headers: ExtrasMap,
        default_query: ExtrasMap,
    ) -> PyResult<Self> {
        let mut builder = reqwest::Client::builder();

        if let Some(headers) = default_headers {
            let headermap = (&headers).try_into().expect("turn to headerrmap");
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
    async fn native_embeddings_create(
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
            .await
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(completion)
    }

    #[pyo3(signature = (model, messages, stream=None, extra_headers=None, extra_query=None, extra=None))]
    async fn native_chat_completions_create(
        &self,
        model: String,
        messages: Messages,
        stream: Option<bool>,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
        extra: Option<Vec<u8>>,
    ) -> PyResult<CompletionResponse> {
        let extra = extra
            .as_deref()
            .map(serde_json::from_slice::<Value>)
            .transpose()
            .expect("Serialize bytes to json");

        let prompt = ChatPrompt::new(model, &messages, stream, extra);

        let completion = self
            .chat_completion(&prompt, extra_headers, extra_query)
            .await
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(completion)
    }
}
