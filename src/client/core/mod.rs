mod handler;
mod mode;

pub use handler::{AsyncExecutor, Executor, Handler};
pub use mode::{Async, Mode, Sync};
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
    pub(crate) client: M::Client,
    pub(crate) api_key: Option<String>,
    pub(crate) base_url: Url,
    pub(crate) provisioning_key: Option<String>,
}

impl<M: Mode> Default for OrpheusCore<M> {
    fn default() -> Self {
        Self {
            client: M::client(),
            api_key: None,
            base_url: Url::parse(DEFAULT_BASE_URL).expect("Default is valid Url"),
            provisioning_key: None,
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

    /// Set the provisioning key that will be used for authorization to the API;
    /// It CANNOT be used to make completion request, only provisioning requests.
    pub fn with_provisioning_key(mut self, provisioning_key: impl Into<String>) -> Self {
        self.provisioning_key = Some(provisioning_key.into());
        self
    }

    pub(crate) fn create_handler<H: Handler<M>>(&self) -> H {
        Handler::from(self)
    }
}
