mod keystore;
mod pool;

use std::collections::HashMap;

use bon::bon;
use keystore::KeyStore;
pub(crate) use pool::Pool;
use secrecy::SecretString;
use url::Url;

use crate::{Result, client::mode::Mode, constants::*};

/// Core client for the Open Responses API.
///
/// To initialize a proper client, use either `Orpheus` or `AsyncOrpheus`.
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
    pub fn new(api_key: impl Into<SecretString>) -> Self {
        Self::builder().api_key(api_key).build()
    }

    /// Initialize an orpheus client with an API key from the environment.
    ///
    /// Valid env vars: ORPHEUS_API_KEY, ORPHEUS_BASE_URL
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var(API_KEY_ENV_VAR)?;

        let base_url = if let Ok(url) = std::env::var(BASE_URL_ENV_VAR) {
            Url::parse(&url)?
        } else {
            Url::parse(DEFAULT_BASE_URL).expect("Default is valid Url")
        };

        Ok(Self::builder()
            .api_key(api_key)
            .base_url(base_url)
            .build())
    }
}

#[bon]
impl<M: Mode> OrpheusCore<M> {
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
        #[builder(default = Url::parse(DEFAULT_BASE_URL).expect("Default is valid Url"))] base_url: Url,
        api_key: Option<SecretString>,
    ) -> Self {
        let keystore = KeyStore::builder()
            .maybe_api_key(api_key)
            .build();

        let pool = pool::Pool::<M>::new(base_url, headers);

        Self { pool, keystore }
    }
}

impl<M: Mode, S: orpheus_core_builder::State> OrpheusCoreBuilder<M, S> {
    pub fn add_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}
