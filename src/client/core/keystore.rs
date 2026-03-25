use bon::Builder;
use secrecy::{ExposeSecret, SecretString};

use crate::{Error, Result};

#[derive(Debug, Clone, Default, Builder)]
pub struct KeyStore {
    api_key: Option<SecretString>,
}

impl KeyStore {
    pub(crate) fn get_api_key(&self) -> Result<&str> {
        let secret = self
            .api_key
            .as_ref()
            .ok_or_else(Error::missing_api_key)?
            .expose_secret();

        Ok(secret)
    }
}
