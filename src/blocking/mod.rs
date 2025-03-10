#![allow(clippy::too_many_arguments)]

mod chat;
mod embed;

use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use pyo3::exceptions::PyKeyError;
use pyo3::prelude::*;
use reqwest::blocking::{Client, Response};
use reqwest::{Error, Method};
use serde::Serialize;

use crate::{API_KEY_ENV, BASE_URL_ENV};

trait SyncRest {
    fn api_request<T: Serialize>(
        &self,
        client: &Client,
        base_url: &url::Url,
        api_key: &str,
        path: &str,
        prompt: &T,
        extra_headers: Option<HashMap<String, String>>,
        extra_query: Option<HashMap<String, String>>,
    ) -> Result<Response, Error> {
        let mut url = base_url.to_owned();

        url.path_segments_mut()
            .expect("get path segments")
            .pop_if_empty()
            .extend(path.split('/').filter(|s| !s.is_empty()));

        if let Some(headers) = extra_query {
            url.query_pairs_mut().extend_pairs(headers);
        };

        let mut builder = client
            .request(Method::POST, url)
            .header("Content-Type", "application/json")
            .bearer_auth(api_key);

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

#[pyclass]
pub struct Orpheus {
    client: Arc<Client>,
    #[pyo3(get)]
    chat: Py<chat::SyncChat>,
    #[pyo3(get)]
    embeddings: Py<embed::SyncEmbed>,
}

#[pymethods]
impl Orpheus {
    #[new]
    #[pyo3(signature = (base_url=None, api_key=None, default_headers=None, default_query=None))]
    fn __init__(
        py: Python<'_>,
        base_url: Option<String>,
        api_key: Option<String>,
        default_headers: Option<HashMap<String, String>>,
        default_query: Option<HashMap<String, String>>,
    ) -> PyResult<Self> {
        let mut builder = Client::builder();

        if let Some(headers) = default_headers {
            let headermap = (&headers).try_into().expect("valid headers");
            builder = builder.default_headers(headermap);
        }

        let client = builder.build().expect("should build http client");

        let mut base_url = base_url
            .or_else(|| env::var(BASE_URL_ENV).ok())
            .and_then(|s| url::Url::parse(&s).ok())
            .ok_or(PyKeyError::new_err(format!(
                "{} environment variable not found.",
                BASE_URL_ENV
            )))
            .expect("should parse base url");

        if let Some(params) = default_query {
            base_url.query_pairs_mut().extend_pairs(params);
        };

        let api_key = api_key
            .or_else(|| env::var(API_KEY_ENV).ok())
            .ok_or(PyKeyError::new_err(format!(
                "{} environment variable not found.",
                API_KEY_ENV
            )))
            .expect("should get api key");

        let client = Arc::new(client);

        let chat = chat::SyncChat::new(Arc::clone(&client), base_url.clone(), api_key.clone());

        let embeddings =
            embed::SyncEmbed::new(Arc::clone(&client), base_url.clone(), api_key.clone());

        Ok(Self {
            client,
            chat: Py::new(py, chat)?,
            embeddings: Py::new(py, embeddings)?,
        })
    }
}
