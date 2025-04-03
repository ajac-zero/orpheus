pub mod chat;
mod common;
mod embed;

use chat::{CompletionResponse, SyncChat};
use common::{Params, SyncRest};
use embed::SyncEmbed;
use pyo3::{exceptions::PyValueError, prelude::*};
use reqwest::blocking;
use serde_json::Value;

use crate::{
    build_client,
    constants::USER_AGENT_NAME,
    models::{
        chat::{message::Messages, prompt::ChatPrompt},
        embed::{EmbeddingInput, EmbeddingPrompt, EmbeddingResponse},
    },
    types::ExtrasMap,
    utils::{get_api_key, get_base_url},
};

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
        let client = build_client!(blocking, default_headers)?;

        let base_url = get_base_url(base_url, default_query)?;

        let api_key = get_api_key(api_key)?;

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
