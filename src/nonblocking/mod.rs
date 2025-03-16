#![allow(clippy::too_many_arguments)]

mod chat;
mod embed;

use std::env;

use pyo3::exceptions::PyKeyError;
use pyo3::prelude::*;

use crate::types::ExtrasMap;
use crate::{API_KEY_ENV, BASE_URL_ENV};

trait AsyncRest {
    async fn api_request<T: serde::Serialize>(
        &self,
        client: &reqwest::Client,
        base_url: &url::Url,
        api_key: &str,
        path: &str,
        prompt: &T,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut url = base_url.to_owned();

        url.path_segments_mut()
            .expect("get path segments")
            .pop_if_empty()
            .extend(path.split('/').filter(|s| !s.is_empty()));

        if let Some(headers) = extra_query {
            url.query_pairs_mut().extend_pairs(headers);
        };

        let mut builder = client
            .request(reqwest::Method::POST, url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key));

        if let Some(headers) = extra_headers {
            builder = headers
                .into_iter()
                .fold(builder, |builder, (k, v)| builder.header(k, v));
        };

        let body = serde_json::to_vec(&prompt).expect("should serialize prompt");

        pyo3_async_runtimes::tokio::get_runtime()
            .spawn(builder.body(body).send())
            .await
            .expect("spawn rt")
    }
}

#[pyclass]
pub struct AsyncOrpheus {
    client: reqwest::Client,
    #[pyo3(get)]
    chat: Py<chat::AsyncChat>,
    #[pyo3(get)]
    embeddings: Py<embed::AsyncEmbed>,
}

#[pymethods]
impl AsyncOrpheus {
    #[new]
    #[pyo3(signature = (base_url=None, api_key=None, default_headers=None, default_query=None))]
    fn __init__(
        py: Python<'_>,
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

        let chat = chat::AsyncChat::new(client.clone(), base_url.clone(), api_key.clone());

        let embeddings = embed::AsyncEmbed::new(client.clone(), base_url.clone(), api_key.clone());

        Ok(Self {
            client,
            chat: Py::new(py, chat)?,
            embeddings: Py::new(py, embeddings)?,
        })
    }
}
