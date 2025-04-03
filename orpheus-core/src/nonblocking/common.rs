use pyo3_async_runtimes::tokio::get_runtime;

use crate::types::ExtrasMap;

pub struct Params {
    client: reqwest::Client,
    url: url::Url,
    key: String,
}

impl Params {
    pub fn new(client: reqwest::Client, url: url::Url, key: String) -> Self {
        Self { client, url, key }
    }
}

pub trait AsyncRest {
    fn get_params(&self) -> &Params;

    async fn api_request<T: serde::Serialize>(
        &self,
        path: &str,
        prompt: &T,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
    ) -> Result<reqwest::Response, reqwest::Error> {
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
            .request(reqwest::Method::POST, url)
            .header("Content-Type", "application/json")
            .bearer_auth(params.key.as_str());

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
