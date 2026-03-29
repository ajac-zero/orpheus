use std::collections::HashMap;

use secrecy::SecretString;
use url::Url;

use crate::{Result, backend::Backend};

const DEFAULT_BASE_URL: &str = open_responses::client::DEFAULT_BASE_URL;
const BASE_URL_ENV_VAR: &str = "ORPHEUS_BASE_URL";
const API_KEY_ENV_VAR: &str = "ORPHEUS_API_KEY";

/// Core client for AI API interactions.
///
/// Generic over backend implementations to support multiple AI providers.
///
/// # Example
/// ```
/// use orpheus::prelude::*;
///
/// let client = Orpheus::new("your_api_key");
/// let async_client = AsyncOrpheus::new("your_api_key");
/// ```
pub struct OrpheusCore<B: Backend> {
    pub(crate) backend: B,
}

impl<M> Default for OrpheusCore<crate::backend::OpenResponsesBackend<M>>
where
    M: open_responses::client::Mode,
    crate::backend::OpenResponsesBackend<M>: Backend,
{
    fn default() -> Self {
        Self {
            backend: crate::backend::OpenResponsesBackend::builder().build(),
        }
    }
}

impl<M> OrpheusCore<crate::backend::OpenResponsesBackend<M>>
where
    M: open_responses::client::Mode,
    crate::backend::OpenResponsesBackend<M>: Backend,
{
    /// Create a new Orpheus client with the provided key.
    pub fn new(api_key: impl Into<SecretString>) -> Self {
        Self {
            backend: crate::backend::OpenResponsesBackend::new(api_key),
        }
    }

    /// Initialize an orpheus client with an API key from the environment.
    ///
    /// Valid env vars: ORPHEUS_API_KEY, ORPHEUS_BASE_URL
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var(API_KEY_ENV_VAR)?;

        let base_url = if let Ok(url) = std::env::var(BASE_URL_ENV_VAR) {
            Url::parse(&url)?
        } else {
            Url::parse(DEFAULT_BASE_URL).expect("default base URL is valid")
        };

        Ok(Self {
            backend: crate::backend::OpenResponsesBackend::builder()
                .api_key(api_key)
                .base_url(base_url)
                .build(),
        })
    }

}

/// Builder for `OrpheusCore` backed by `OpenResponsesBackend`.
pub struct OrpheusCoreBuilder<M: open_responses::client::Mode> {
    headers: HashMap<String, String>,
    base_url: Url,
    api_key: Option<SecretString>,
    _mode: std::marker::PhantomData<M>,
}

impl<M> OrpheusCoreBuilder<M>
where
    M: open_responses::client::Mode,
    crate::backend::OpenResponsesBackend<M>: Backend,
{
    pub fn api_key(mut self, key: impl Into<SecretString>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn base_url(mut self, url: Url) -> Self {
        self.base_url = url;
        self
    }

    pub fn add_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> OrpheusCore<crate::backend::OpenResponsesBackend<M>> {
        let b = crate::backend::OpenResponsesBackend::<M>::builder()
            .maybe_api_key(self.api_key)
            .base_url(self.base_url);
        let backend = self.headers.into_iter().fold(b, |b, (k, v)| b.add_header(k, v)).build();
        OrpheusCore { backend }
    }
}

impl<M> OrpheusCore<crate::backend::OpenResponsesBackend<M>>
where
    M: open_responses::client::Mode,
    crate::backend::OpenResponsesBackend<M>: Backend,
{
    /// Initialize an Orpheus builder to customize the client.
    ///
    /// # Example
    /// ```rust
    /// use orpheus::prelude::*;
    ///
    /// let client = Orpheus::builder()
    ///     .add_header("X-Custom-Header", "Custom Value")
    ///     .api_key("your_api_key")
    ///     .base_url(url::Url::parse("https://api.example.com/v1").expect("Valid Url"))
    ///     .build();
    /// ```
    pub fn builder() -> OrpheusCoreBuilder<M> {
        OrpheusCoreBuilder {
            headers: HashMap::new(),
            base_url: Url::parse(DEFAULT_BASE_URL).expect("default base URL is valid"),
            api_key: None,
            _mode: std::marker::PhantomData,
        }
    }
}
