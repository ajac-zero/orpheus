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

// ── OpenResponsesBackend (sync) ───────────────────────────────────────────────

impl Default for OrpheusCore<crate::backend::OpenResponsesBackend> {
    fn default() -> Self {
        Self {
            backend: crate::backend::OpenResponsesBackend::builder().build(),
        }
    }
}

impl OrpheusCore<crate::backend::OpenResponsesBackend> {
    /// Create a new Orpheus client with the provided key.
    pub fn new(api_key: impl Into<SecretString>) -> Self {
        Self {
            backend: crate::backend::OpenResponsesBackend::new(api_key),
        }
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
        Ok(Self {
            backend: crate::backend::OpenResponsesBackend::builder()
                .api_key(api_key)
                .base_url(base_url)
                .build(),
        })
    }

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
    pub fn builder() -> OrpheusCoreBuilder {
        OrpheusCoreBuilder {
            headers: HashMap::new(),
            base_url: Url::parse(DEFAULT_BASE_URL).expect("default base URL is valid"),
            api_key: None,
        }
    }
}

pub struct OrpheusCoreBuilder {
    headers: HashMap<String, String>,
    base_url: Url,
    api_key: Option<SecretString>,
}

impl OrpheusCoreBuilder {
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

    pub fn build(self) -> OrpheusCore<crate::backend::OpenResponsesBackend> {
        let b = crate::backend::OpenResponsesBackend::builder()
            .maybe_api_key(self.api_key)
            .base_url(self.base_url);
        let backend = self
            .headers
            .into_iter()
            .fold(b, |b, (k, v)| b.add_header(k, v))
            .build();
        OrpheusCore { backend }
    }
}

// ── AsyncOpenResponsesBackend ─────────────────────────────────────────────────

impl Default for OrpheusCore<crate::backend::AsyncOpenResponsesBackend> {
    fn default() -> Self {
        Self {
            backend: crate::backend::AsyncOpenResponsesBackend::builder().build(),
        }
    }
}

impl OrpheusCore<crate::backend::AsyncOpenResponsesBackend> {
    /// Create a new async Orpheus client with the provided key.
    pub fn new(api_key: impl Into<SecretString>) -> Self {
        Self {
            backend: crate::backend::AsyncOpenResponsesBackend::new(api_key),
        }
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
        Ok(Self {
            backend: crate::backend::AsyncOpenResponsesBackend::builder()
                .api_key(api_key)
                .base_url(base_url)
                .build(),
        })
    }

    /// Initialize an async Orpheus builder to customize the client.
    ///
    /// # Example
    /// ```rust
    /// use orpheus::prelude::*;
    ///
    /// let client = AsyncOrpheus::builder()
    ///     .add_header("X-Custom-Header", "Custom Value")
    ///     .api_key("your_api_key")
    ///     .base_url(url::Url::parse("https://api.example.com/v1").expect("Valid Url"))
    ///     .build();
    /// ```
    pub fn builder() -> AsyncOrpheusCoreBuilder {
        AsyncOrpheusCoreBuilder {
            headers: HashMap::new(),
            base_url: Url::parse(DEFAULT_BASE_URL).expect("default base URL is valid"),
            api_key: None,
        }
    }
}

pub struct AsyncOrpheusCoreBuilder {
    headers: HashMap<String, String>,
    base_url: Url,
    api_key: Option<SecretString>,
}

impl AsyncOrpheusCoreBuilder {
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

    pub fn build(self) -> OrpheusCore<crate::backend::AsyncOpenResponsesBackend> {
        let b = crate::backend::AsyncOpenResponsesBackend::builder()
            .maybe_api_key(self.api_key)
            .base_url(self.base_url);
        let backend = self
            .headers
            .into_iter()
            .fold(b, |b, (k, v)| b.add_header(k, v))
            .build();
        OrpheusCore { backend }
    }
}

// ── GeminiBackend ─────────────────────────────────────────────────────────────

#[cfg(feature = "gemini")]
impl<M> OrpheusCore<crate::backend::GeminiBackend<M>>
where
    crate::backend::GeminiBackend<M>: Backend,
{
    pub fn new(api_key: impl Into<SecretString>) -> Self {
        Self {
            backend: crate::backend::GeminiBackend::new(api_key),
        }
    }

    pub fn from_env() -> Result<Self> {
        Ok(Self {
            backend: crate::backend::GeminiBackend::from_env()?,
        })
    }

    pub fn builder() -> GeminiOrpheusCoreBuilder<M> {
        GeminiOrpheusCoreBuilder {
            headers: HashMap::new(),
            base_url: Url::parse(crate::backend::gemini::DEFAULT_BASE_URL)
                .expect("default Gemini base URL is valid"),
            api_key: None,
            _mode: std::marker::PhantomData,
        }
    }
}

#[cfg(feature = "gemini")]
pub struct GeminiOrpheusCoreBuilder<M> {
    headers: HashMap<String, String>,
    base_url: Url,
    api_key: Option<SecretString>,
    _mode: std::marker::PhantomData<M>,
}

#[cfg(feature = "gemini")]
impl<M> GeminiOrpheusCoreBuilder<M>
where
    crate::backend::GeminiBackend<M>: Backend,
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

    pub fn build(self) -> OrpheusCore<crate::backend::GeminiBackend<M>> {
        let b = crate::backend::GeminiBackend::<M>::builder()
            .maybe_api_key(self.api_key)
            .base_url(self.base_url);
        let backend = self
            .headers
            .into_iter()
            .fold(b, |b, (k, v)| b.add_header(k, v))
            .build();
        OrpheusCore { backend }
    }
}
