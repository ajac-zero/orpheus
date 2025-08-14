use reqwest::header::CONTENT_TYPE;
use url::Url;

use crate::{
    Error, Result,
    constants::*,
    models::common::{
        handler::{AsyncExecutor, Executor},
        mode::{Async, Mode, Sync},
    },
};

/// Client to interface with LLMs;
/// Follows the OpenAI API specification.
#[derive(Debug, Clone)]
pub struct OrpheusCore<M: Mode> {
    client: M::Client,
    api_key: Option<String>,
    base_url: Url,
}

impl<M: Mode> Default for OrpheusCore<M> {
    fn default() -> Self {
        Self {
            client: M::client(),
            api_key: None,
            base_url: Url::parse(DEFAULT_BASE_URL).expect("Default is valid Url"),
        }
    }
}

impl<M: Mode> OrpheusCore<M> {
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

    /// Set the base URL for the API
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }
}

impl OrpheusCore<Sync> {
    pub fn create_handler<H: Executor>(&self) -> H {
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
}

impl OrpheusCore<Async> {
    pub fn create_handler<H: AsyncExecutor>(&self) -> H {
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
}

pub type Orpheus = OrpheusCore<Sync>;
pub type AsyncOrpheus = OrpheusCore<Async>;

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_async_client_creation() {
        let client = AsyncOrpheus::new("test_key");
        assert_eq!(
            client.base_url,
            Url::parse("https://openrouter.ai/api/v1/").unwrap()
        );
        assert_eq!(client.api_key, Some("test_key".to_string()));
    }

    #[test]
    fn test_async_client_with_base_url() {
        let client = AsyncOrpheus::new("test_key")
            .with_base_url("https://custom-api.example.com/v1")
            .unwrap();
        assert_eq!(
            client.base_url,
            Url::parse("https://custom-api.example.com/v1").unwrap()
        );
    }
}
