mod chat;
mod embed;

use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::get_runtime;

use crate::{
    build_client,
    constants::USER_AGENT_NAME,
    utils::{get_api_key, get_base_url},
};

#[pyclass(frozen, subclass)]
pub struct AsyncOrpheusCore {
    client: reqwest::Client,
    url: url::Url,
    key: String,
}

#[pymethods]
impl AsyncOrpheusCore {
    #[new]
    #[pyo3(signature = (base_url=None, api_key=None, default_headers=None, default_query=None))]
    fn new(
        base_url: Option<String>,
        api_key: Option<String>,
        default_headers: Option<HashMap<String, String>>,
        default_query: Option<HashMap<String, String>>,
    ) -> PyResult<Self> {
        Ok(Self {
            client: build_client!(reqwest, default_headers)?,
            url: get_base_url(base_url, default_query)?,
            key: get_api_key(api_key)?,
        })
    }
}

impl AsyncOrpheusCore {
    async fn api_request<T: serde::Serialize>(
        &self,
        path: &str,
        prompt: &T,
        extra_headers: Option<HashMap<String, String>>,
        extra_query: Option<HashMap<String, String>>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut url = self.url.to_owned();

        url.path_segments_mut()
            .expect("get path segments")
            .pop_if_empty()
            .extend(path.split('/').filter(|s| !s.is_empty()));

        if let Some(headers) = extra_query {
            url.query_pairs_mut().extend_pairs(headers);
        };

        let mut builder = self
            .client
            .request(reqwest::Method::POST, url)
            .header("Content-Type", "application/json")
            .bearer_auth(self.key.as_str());

        if let Some(headers) = extra_headers {
            builder = headers
                .into_iter()
                .fold(builder, |builder, (k, v)| builder.header(k, v));
        };

        let body = serde_json::to_vec(&prompt).expect("should serialize prompt");

        get_runtime()
            .spawn(builder.body(body).send())
            .await
            .expect("spawn task within runtime")
    }
}
