use std::collections::HashMap;

use bon::bon;
use secrecy::SecretString;
use url::Url;

use crate::Result;

const DEFAULT_BASE_URL: &str = open_responses::client::DEFAULT_BASE_URL;
const BASE_URL_ENV_VAR: &str = "ORPHEUS_BASE_URL";
const API_KEY_ENV_VAR: &str = "ORPHEUS_API_KEY";

/// Blocking Orpheus client for the Open Responses API.
///
/// # Example
/// ```
/// use orpheus::prelude::*;
///
/// let client = Orpheus::new("your_api_key");
/// ```
pub struct Orpheus {
    pub(crate) inner: open_responses::client::Client,
}

/// Async Orpheus client for the Open Responses API.
///
/// # Example
/// ```
/// use orpheus::prelude::*;
///
/// let client = AsyncOrpheus::new("your_api_key");
/// ```
pub struct AsyncOrpheus {
    pub(crate) inner: open_responses::client::AsyncClient,
}

impl Default for Orpheus {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl Orpheus {
    /// Create a new Orpheus client with the provided key.
    pub fn new(api_key: impl Into<SecretString>) -> Self {
        Self::builder().api_key(api_key).build()
    }

    /// Initialize an Orpheus client with an API key from the environment.
    ///
    /// Valid env vars: ORPHEUS_API_KEY, ORPHEUS_BASE_URL
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var(API_KEY_ENV_VAR)?;

        let base_url = if let Ok(url) = std::env::var(BASE_URL_ENV_VAR) {
            Url::parse(&url)?
        } else {
            Url::parse(DEFAULT_BASE_URL).expect("default base URL is valid")
        };

        Ok(Self::builder().api_key(api_key).base_url(base_url).build())
    }
}

#[bon]
impl Orpheus {
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
    #[builder(on(SecretString, into))]
    pub fn builder(
        #[builder(field)] headers: HashMap<String, String>,
        #[builder(default = Url::parse(DEFAULT_BASE_URL).expect("default base URL is valid"))]
        base_url: Url,
        api_key: Option<SecretString>,
    ) -> Self {
        let builder = open_responses::client::Client::builder()
            .maybe_api_key(api_key)
            .base_url(base_url);
        let builder = headers.into_iter().fold(builder, |builder, (key, value)| {
            builder.add_header(key, value)
        });

        Self {
            inner: builder.build(),
        }
    }
}

impl<S: orpheus_builder::State> OrpheusBuilder<S> {
    pub fn add_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

impl Default for AsyncOrpheus {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl AsyncOrpheus {
    /// Create a new async Orpheus client with the provided key.
    pub fn new(api_key: impl Into<SecretString>) -> Self {
        Self::builder().api_key(api_key).build()
    }

    /// Initialize an async Orpheus client with an API key from the environment.
    ///
    /// Valid env vars: ORPHEUS_API_KEY, ORPHEUS_BASE_URL
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var(API_KEY_ENV_VAR)?;

        let base_url = if let Ok(url) = std::env::var(BASE_URL_ENV_VAR) {
            Url::parse(&url)?
        } else {
            Url::parse(DEFAULT_BASE_URL).expect("default base URL is valid")
        };

        Ok(Self::builder().api_key(api_key).base_url(base_url).build())
    }
}

#[bon]
impl AsyncOrpheus {
    /// Initialize an async Orpheus builder to customize the client.
    #[builder(on(SecretString, into))]
    pub fn builder(
        #[builder(field)] headers: HashMap<String, String>,
        #[builder(default = Url::parse(DEFAULT_BASE_URL).expect("default base URL is valid"))]
        base_url: Url,
        api_key: Option<SecretString>,
    ) -> Self {
        let builder = open_responses::client::AsyncClient::builder()
            .maybe_api_key(api_key)
            .base_url(base_url);
        let builder = headers.into_iter().fold(builder, |builder, (key, value)| {
            builder.add_header(key, value)
        });

        Self {
            inner: builder.build(),
        }
    }
}

impl<S: async_orpheus_builder::State> AsyncOrpheusBuilder<S> {
    pub fn add_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}
