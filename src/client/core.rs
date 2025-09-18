mod keystore;
mod pool;

use std::collections::HashMap;

use bon::bon;
use keystore::KeyStore;
pub(crate) use pool::Pool;
use secrecy::SecretString;
use url::Url;

use crate::{Result, client::mode::Mode, constants::*};

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
pub struct OrpheusCore<M: Mode> {
    pub(crate) pool: Pool<M>,
    pub(crate) keystore: KeyStore,
}

impl<M: Mode> Default for OrpheusCore<M> {
    fn default() -> Self {
        Self::builder().build()
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
    pub fn new(api_key: impl Into<SecretString>) -> Self {
        Self::builder().api_key(api_key).build()
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
        let api_key = std::env::var(API_KEY_ENV_VAR)?;
        Ok(Self::new(api_key))
    }
}

#[bon]
impl<M: Mode> OrpheusCore<M> {
    /// Initialize an Orpheus builder to customize the client.
    ///
    /// # Example
    /// ```rust
    ///use orpheus::prelude::*;
    ///
    /// let client = Orpheus::builder()
    ///     .add_header("X-Custom-Header", "Custom Value") // You can add as many headers as you want
    ///     .x_title("My App")
    ///     .http_referer("https://my-app.com")
    ///     .api_key("your_api_key")
    ///     .provisioning_key("your_provisioning_key")
    ///     .base_url(url::Url::parse("https://your-base-url.com").expect("Valid Url"))
    ///     .build();
    /// ```
    #[builder(on(SecretString, into))]
    pub fn builder(
        #[builder(field)] mut headers: HashMap<String, String>,
        #[builder(default = Url::parse(DEFAULT_BASE_URL).expect("Default is valid Url"))] base_url: Url,
        api_key: Option<SecretString>,
        provisioning_key: Option<SecretString>,
    ) -> Self {
        if headers.get("X-Title").is_none() {
            headers.insert("X-Title".into(), "Orpheus".into());
        }

        if headers.get("HTTP-Referer").is_none() {
            headers.insert(
                "HTTP-Referer".into(),
                "https://orpheus.ajac-zero.com".into(),
            );
        }

        let keystore = KeyStore::builder()
            .maybe_api_key(api_key)
            .maybe_provisioning_key(provisioning_key)
            .build();

        let pool = pool::Pool::<M>::new(base_url);

        Self { pool, keystore }
    }
}

impl<M: Mode, S: orpheus_core_builder::State> OrpheusCoreBuilder<M, S> {
    pub fn add_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn x_title(mut self, title: impl Into<String>) -> Self {
        self.headers.insert("X-Title".into(), title.into());
        self
    }

    pub fn http_referer(mut self, referer: impl Into<String>) -> Self {
        self.headers.insert("HTTP-Referer".into(), referer.into());
        self
    }
}
