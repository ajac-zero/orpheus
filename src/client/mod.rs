mod blocking;
mod non_blocking;

pub use blocking::Orpheus;
pub use non_blocking::AsyncOrpheus;

use super::types::{Completion, CompletionChunk, Prompt};
use isahc::{HttpClient, Request};
use pyo3::exceptions::PyKeyError;
use std::collections::HashMap;
use std::env;
use url::Url;

const BASE_URL_ENV: &str = "ORPHEUS_BASE_URL";
const API_KEY_ENV: &str = "ORPHEUS_API_KEY";

fn build_request(
    path: &str,
    prompt: &Prompt,
    url: &Url,
    api_key: &str,
    extra_headers: Option<HashMap<String, String>>,
    extra_query: Option<HashMap<String, String>>,
) -> Request<Vec<u8>> {
    let mut url = url.to_owned();

    url.path_segments_mut()
        .expect("get path segments")
        .pop_if_empty()
        .extend(path.split('/').filter(|s| !s.is_empty()));

    if let Some(headers) = extra_query {
        url.query_pairs_mut().extend_pairs(headers);
    };

    let mut builder = Request::builder()
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

    builder.body(body).expect("should build request")
}

fn new(
    base_url: Option<String>,
    api_key: Option<String>,
    default_headers: Option<HashMap<String, String>>,
    default_query: Option<HashMap<String, String>>,
) -> (HttpClient, Url, String) {
    let mut builder = HttpClient::builder();

    if let Some(headers) = default_headers {
        builder = builder.default_headers(headers);
    }

    let client = builder.build().expect("should build http client");

    let mut base_url = base_url
        .or_else(|| env::var(BASE_URL_ENV).ok())
        .and_then(|s| Url::parse(&s).ok())
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

    (client, base_url, api_key)
}
