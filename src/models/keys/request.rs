use bon::{Builder, builder};
use key_provisioning_request_builder::{IsSet, IsUnset, State};
use serde::Serialize;

use crate::{
    Error, Result,
    client::{Async, AsyncExecutor, Executor, Mode, Sync},
    models::keys::{CreateKeyResult, DeleteKeyResult, ListKeysResult, ProvisionHandler},
};

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct KeyProvisioningRequest<M: Mode> {
    #[serde(skip)]
    #[builder(start_fn)]
    handler: Option<ProvisionHandler<M>>,

    #[serde(skip)]
    #[builder(field)]
    pub(crate) method: reqwest::Method,

    pub name: Option<String>,

    pub hash: Option<String>,

    pub label: Option<String>,

    /// Optional credit limit.
    pub limit: Option<usize>,

    /// Disabled the key
    pub disabled: Option<bool>,

    /// Optional: control BYOK usage in limit
    pub include_byok_in_limit: Option<bool>,

    #[serde(skip)]
    pub offset: Option<usize>,
}

impl<S: State> KeyProvisioningRequestBuilder<Sync, S>
where
    S::Name: IsSet,
    S::Disabled: IsUnset,
    S::Offset: IsUnset,
{
    pub fn create(mut self) -> Result<CreateKeyResult> {
        let handler = self.handler.take().expect("Has handler");

        self.method = reqwest::Method::POST;

        let body = self.build();

        let response = handler.execute(body)?;

        Ok(response.json().map_err(Error::http)?)
    }
}

impl<S: State> KeyProvisioningRequestBuilder<Async, S>
where
    S::Name: IsSet,
    S::Disabled: IsUnset,
    S::Offset: IsUnset,
{
    pub async fn create(mut self) -> Result<CreateKeyResult> {
        let handler = self.handler.take().expect("Has handler");

        self.method = reqwest::Method::POST;

        let body = self.build();

        let response = handler.execute(body).await?;

        Ok(response.json().await.map_err(Error::http)?)
    }
}

impl<S: State> KeyProvisioningRequestBuilder<Sync, S> {
    pub fn list(mut self) -> Result<ListKeysResult> {
        let handler = self.handler.take().expect("Has handler");

        self.method = reqwest::Method::GET;

        let body = self.build();

        let response = handler.execute(body)?;

        Ok(response.json().map_err(Error::http)?)
    }
}

impl<S: State> KeyProvisioningRequestBuilder<Sync, S>
where
    S::Hash: IsSet,
    S::Name: IsUnset,
    S::Disabled: IsUnset,
    S::Offset: IsUnset,
    S::IncludeByokInLimit: IsUnset,
{
    pub fn delete(mut self) -> Result<DeleteKeyResult> {
        let handler = self.handler.take().expect("Has handler");

        self.method = reqwest::Method::DELETE;

        let body = self.build();

        let response = handler.execute(body)?;

        Ok(response.json().map_err(Error::http)?)
    }
}
