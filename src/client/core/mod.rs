mod handler;
mod mode;

pub use handler::{AsyncExecutor, Executor, Handler};
pub use mode::{Async, Mode, Sync};
use reqwest::header::CONTENT_TYPE;
use url::Url;

use crate::{Error, Result, constants::*};

/// Core client logic to interface with LLMs.
/// Designed for the OpenRouter API, but
/// follows the OpenAI API specification.
///
/// To initialize a proper client, you need to use either `Orpheus` or `AsyncOrpheus`.
///
/// # Example
/// ```
/// use orpheus::prelude::*;
///
/// let client = Orpheus::new("your_api_key");
/// let async_client = AsyncOrpheus::new("your_api_key");
/// ```
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
    /// Create a new Orpheus client with the provided key.
    ///
    /// # Example
    /// ```
    /// use orpheus::prelude::*;
    ///
    /// let client = Orpheus::new("your_api_key");
    /// let async_client = AsyncOrpheus::new("your_api_key");
    /// ```
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
    /// let async_client = AsyncOrpheus::from_env().expect("ORPHEUS_API_KEY is set");
    /// ```
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var(API_KEY_ENV_VAR).map_err(Error::env)?;
        Ok(Self::new(api_key))
    }

    /// Set the base URL for all client requests;
    /// Will work with any OpenAI-compatible endpoint.
    ///
    /// # Example
    /// ```
    /// use orpheus::prelude::*;
    ///
    /// let client = Orpheus::new("your_api_key").with_base_url("https://api.example.com").expect("Is valid url");
    /// ```
    pub fn with_base_url(
        mut self,
        base_url: impl TryInto<Url, Error = url::ParseError>,
    ) -> Result<Self> {
        self.base_url = base_url.try_into().map_err(Error::invalid_url)?;
        Ok(self)
    }

    /// Set the API key that will be used for authorization to the API;
    /// The API key will be used in a Bearer Authorization header.
    ///
    /// # Example
    /// ```
    /// use orpheus::prelude::*;
    ///
    /// // `Default::default` for `OrpheusCore<_>` creates a client without authorization.
    /// let client = Orpheus::default().with_api_key("your_api_key");
    /// ```
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }
}

// Macro to implement create_handler for both Sync and Async modes
macro_rules! impl_create_handler {
    ($mode:ty, $trait_bound:path) => {
        impl OrpheusCore<$mode> {
            pub(crate) fn create_handler<H: $trait_bound>(&self) -> H {
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
    };
}

// Apply the macro for both Sync and Async modes
impl_create_handler!(Sync, Executor);
impl_create_handler!(Async, AsyncExecutor);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{AsyncOrpheus, Orpheus};

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
