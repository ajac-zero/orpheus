mod chat;
mod embed;

use std::env;

use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::prelude::*;
use reqwest::blocking::{Client, Response};
use reqwest::{Error, Method};
use serde::Serialize;
use serde_json::Value;

use crate::types::embed::{EmbeddingInput, EmbeddingPrompt, EmbeddingResponse};
use crate::types::message::{EitherMessages, Messages};
use crate::types::prompt::Prompt;
use crate::types::ExtrasMap;
use crate::{API_KEY_ENVS, BASE_URL_ENVS};

use chat::{CompletionResponse, SyncChat};
use embed::SyncEmbed;

struct Params<'a> {
    client: &'a Client,
    url: &'a url::Url,
    key: &'a str,
}

trait SyncRest {
    fn get_params(&self) -> Params;

    fn api_request<T: Serialize>(
        &self,
        path: &str,
        prompt: &T,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
    ) -> Result<Response, Error> {
        let params = self.get_params();

        let mut url = params.url.to_owned();

        url.path_segments_mut()
            .expect("get path segments")
            .pop_if_empty()
            .extend(path.split('/').filter(|s| !s.is_empty()));

        if let Some(headers) = extra_query {
            url.query_pairs_mut().extend_pairs(headers);
        };

        let mut builder = params
            .client
            .request(Method::POST, url)
            .header("Content-Type", "application/json")
            .bearer_auth(params.key);

        if let Some(headers) = extra_headers {
            builder = headers
                .into_iter()
                .fold(builder, |builder, (k, v)| builder.header(k, v));
        };

        let body = serde_json::to_vec(&prompt).expect("should serialize prompt");

        let request = builder.body(body);

        request.send()
    }
}

#[pyclass(frozen, subclass)]
pub struct OrpheusCore {
    client: Client,
    base_url: url::Url,
    api_key: String,
}

impl SyncRest for OrpheusCore {
    fn get_params(&self) -> Params {
        Params {
            client: &self.client,
            url: &self.base_url,
            key: &self.api_key,
        }
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
            client,
            base_url,
            api_key,
        })
    }

    #[pyo3(signature = (input, model, dimensions=None, encoding_format=None, user=None, extra_headers=None, extra_query=None))]
    fn create_embeddings(
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
    fn create_chat_completion(
        &self,
        py: Python,
        model: String,
        messages: EitherMessages,
        stream: Option<bool>,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
        extra: Option<&[u8]>,
    ) -> PyResult<CompletionResponse> {
        let messages = messages.map_left(Ok).left_or_else(|x| {
            x.extract::<Messages>(py)
                .map(|x| Py::new(py, x).expect("bind to GIL"))
        })?;

        let extra = extra
            .map(serde_json::from_slice::<Value>)
            .transpose()
            .expect("Serialize bytes to json");

        let prompt = Prompt::new(model, messages.get(), stream, extra);

        let completion = self
            .chat_completion(&prompt, extra_headers, extra_query)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(completion)
    }
}
