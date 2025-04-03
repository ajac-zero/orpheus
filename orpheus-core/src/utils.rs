use std::{collections::HashMap, env};

use anyhow::{Context, anyhow};
use url::Url;

use crate::constants::{API_KEY_ENVS, BASE_URL_ENVS};

pub fn get_base_url(
    base_url: Option<String>,
    default_query: Option<HashMap<String, String>>,
) -> anyhow::Result<Url> {
    base_url
        .or_else(|| BASE_URL_ENVS.iter().find_map(|s| env::var(s).ok()))
        .ok_or_else(|| {
            anyhow!(
                "No base URL provided and none found in environment variables: {:?}",
                BASE_URL_ENVS
            )
        })
        .and_then(|s| {
            Url::parse(&s)
                .with_context(|| format!("Failed to parse base URL string '{}' as a URL", s))
        })
        .map(|mut url| {
            if let Some(params) = default_query {
                url.query_pairs_mut().extend_pairs(params);
            };
            url
        })
}

pub fn get_api_key(api_key: Option<String>) -> anyhow::Result<String> {
    api_key
        .or_else(|| API_KEY_ENVS.iter().find_map(|s| env::var(s).ok()))
        .ok_or_else(|| {
            anyhow!(
                "No API key provided and none found in environment variables: {:?}",
                API_KEY_ENVS
            )
        })
}

#[macro_export]
macro_rules! build_client {
    ($client_mod:ident, $headers_option:expr) => {{
        use anyhow::Context;
        use reqwest::header::HeaderMap;

        let mut builder = $client_mod::Client::builder();

        if let Some(headers) = $headers_option {
            let headermap: HeaderMap = (&headers)
                .try_into()
                .context("Failed to convert default headers into HeaderMap")?;
            builder = builder.default_headers(headermap);
        }

        let client_result = builder
            .user_agent(USER_AGENT_NAME)
            .use_rustls_tls()
            .build()
            .context("Failed to build reqwest client");

        client_result
    }};
}
