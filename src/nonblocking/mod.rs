#![allow(clippy::too_many_arguments)]

mod chat;
mod embed;

use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use pyo3::exceptions::PyKeyError;
use pyo3::prelude::*;
use serde::Serialize;

use crate::{API_KEY_ENV, BASE_URL_ENV};

trait AsyncRest {
    async fn api_request<T: Serialize>(
        &self,
        client: &isahc::HttpClient,
        base_url: &url::Url,
        api_key: &str,
        path: &str,
        prompt: &T,
        extra_headers: Option<HashMap<String, String>>,
        extra_query: Option<HashMap<String, String>>,
    ) -> Result<isahc::Response<isahc::AsyncBody>, isahc::Error> {
        let mut url = base_url.to_owned();

        url.path_segments_mut()
            .expect("get path segments")
            .pop_if_empty()
            .extend(path.split('/').filter(|s| !s.is_empty()));

        if let Some(headers) = extra_query {
            url.query_pairs_mut().extend_pairs(headers);
        };

        let mut builder = isahc::Request::builder()
            .method("POST")
            .uri(url.as_str())
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key));

        if let Some(headers) = extra_headers {
            builder = headers
                .into_iter()
                .fold(builder, |builder, (k, v)| builder.header(k, v));
        };

        let body = serde_json::to_vec(&prompt).expect("should serialize prompt");

        let request = builder.body(body).expect("should build request");

        client.send_async(request).await
    }
}

#[pyclass]
pub struct AsyncOrpheus {
    client: Arc<isahc::HttpClient>,
    chat: chat::AsyncChat,
    embeddings: embed::AsyncEmbed,
}

#[pymethods]
impl AsyncOrpheus {
    #[new]
    #[pyo3(signature = (base_url=None, api_key=None, default_headers=None, default_query=None))]
    fn __init__(
        base_url: Option<String>,
        api_key: Option<String>,
        default_headers: Option<HashMap<String, String>>,
        default_query: Option<HashMap<String, String>>,
    ) -> PyResult<Self> {
        let mut builder = isahc::HttpClient::builder();

        if let Some(headers) = default_headers {
            builder = builder.default_headers(headers);
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

        let chat = chat::AsyncChat::new(Arc::clone(&client), base_url.clone(), api_key.clone());

        let embeddings =
            embed::AsyncEmbed::new(Arc::clone(&client), base_url.clone(), api_key.clone());

        Ok(Self {
            client,
            chat,
            embeddings,
        })
    }
}
