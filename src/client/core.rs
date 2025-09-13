use std::collections::HashMap;

use bon::bon;
use url::Url;

use crate::{
    Error, Result,
    client::{Handler, Mode},
    constants::*,
};

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
    pub(crate) client: M::Client,
    pub(crate) api_key: Option<String>,
    pub(crate) base_url: Url,
    pub(crate) provisioning_key: Option<String>,
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
    pub fn new(api_key: impl Into<String>) -> Self {
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
        let api_key = std::env::var(API_KEY_ENV_VAR).map_err(Error::env)?;
        Ok(Self::new(api_key))
    }

    pub(crate) fn create_handler<H: Handler<M>>(&self) -> H {
        Handler::from(self)
    }
}

#[bon]
impl<M: Mode> OrpheusCore<M> {
    #[builder(on(String, into))]
    pub fn builder(
        #[builder(field)] mut headers: HashMap<String, String>,
        #[builder(default = Url::parse(DEFAULT_BASE_URL).expect("Default is valid Url"))] base_url: Url,
        api_key: Option<String>,
        provisioning_key: Option<String>,
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

        Self {
            client: M::client(headers),
            base_url,
            api_key,
            provisioning_key,
        }
    }
}

impl<M: Mode, S: orpheus_core_builder::State> OrpheusCoreBuilder<M, S> {
    fn add_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
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
