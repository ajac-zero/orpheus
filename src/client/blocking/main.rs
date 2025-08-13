use reqwest::{blocking::Client, header::CONTENT_TYPE};
use url::Url;

use crate::{Error, Result, constants::*, models::common::handler::Handler};

/// Client to interface with LLMs;
/// Follows the OpenAI API specification.
#[derive(Debug, Clone)]
pub struct Orpheus {
    client: Client,
    api_key: Option<String>,
    base_url: Url,
}

impl Default for Orpheus {
    fn default() -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT_NAME)
            .use_rustls_tls()
            .build()
            .expect("build request client");

        Self {
            client,
            api_key: None,
            base_url: Url::parse(DEFAULT_BASE_URL).expect("Default is valid Url"),
        }
    }
}

impl Orpheus {
    /// Create a new Orpheus client with provided key and default base url
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::default().with_api_key(api_key)
    }

    /// Initialize an orpheus client with an API key
    /// from the environment.
    ///
    /// Valid env vars: ORPHEUS_API_KEY
    ///
    /// # Example
    /// ```
    /// use orpheus::prelude::*;
    ///
    /// let client = Orpheus::from_env().expect("ORPHEUS_API_KEY is set");
    /// ```
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var(API_KEY_ENV_VAR).map_err(Error::env)?;
        Ok(Self::new(api_key))
    }

    /// Set the base URL for the API
    pub fn with_base_url(
        mut self,
        base_url: impl TryInto<Url, Error = url::ParseError>,
    ) -> Result<Self> {
        self.base_url = base_url.try_into().map_err(Error::invalid_url)?;
        Ok(self)
    }

    pub fn execute(
        &self,
        path: &str,
        body: impl serde::Serialize,
    ) -> Result<reqwest::blocking::Response> {
        let url = self.base_url.join(path).map_err(Error::invalid_url)?;
        let token = self
            .api_key
            .as_ref()
            .map_or_else(String::new, |key| key.clone());
        let response = self
            .client
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .bearer_auth(token)
            .json(&body)
            .send()
            .map_err(Error::http)?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let err = response.text().map_err(Error::http)?;
            Err(Error::openrouter(err))
        }
    }

    pub fn create_handler<H: Handler>(&self) -> H {
        let url = self.base_url.join(H::PATH).expect("Is valid url");
        let mut builder = self
            .client
            .post(url)
            .header(CONTENT_TYPE, "application/json");

        if let Some(token) = self.api_key.as_ref() {
            builder = builder.bearer_auth(token);
        }

        H::new(builder)
    }

    /// Set the base URL for the API
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::Orpheus;

    #[test]
    fn test_client_creation() {
        let client = Orpheus::new("test_key");
        assert_eq!(
            client.base_url,
            Url::parse("https://openrouter.ai/api/v1/").unwrap()
        );
        assert_eq!(client.api_key, Some("test_key".to_string()));
    }

    #[test]
    fn test_client_with_base_url() {
        let client = Orpheus::new("test_key")
            .with_base_url("https://custom-api.example.com/v1")
            .unwrap();
        assert_eq!(
            client.base_url,
            Url::parse("https://custom-api.example.com/v1").unwrap()
        );
    }
}
