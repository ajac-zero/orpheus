use bon::{Builder, builder};
use key_provisioning_request_builder::{IsSet, IsUnset, State};
use serde::Serialize;

use crate::{
    Result,
    client::{
        core::Pool,
        mode::{Async, Mode, Sync},
    },
    constants::KEY_PROVISION_PATH,
    models::keys::{CreateKeyResult, DeleteKeyResult, ListKeysResult},
};

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Builder)]
#[builder(on(String, into))]
pub struct KeyProvisioningRequest<'a, M: Mode> {
    #[serde(skip)]
    #[builder(start_fn)]
    pool: &'a Pool<M>,

    #[serde(skip)]
    #[builder(start_fn)]
    provisioning_key: String,

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

impl<'a, S: State> KeyProvisioningRequestBuilder<'a, Sync, S>
where
    S::Name: IsSet,
    S::Disabled: IsUnset,
    S::Offset: IsUnset,
{
    pub fn create(self) -> Result<CreateKeyResult> {
        let mut handler = self.pool.get().expect("Has handler");

        let token = self.provisioning_key.clone();
        let body = self.build();

        let response = handler
            .execute()
            .segments(&[KEY_PROVISION_PATH])
            .method("POST")
            .payload(body)
            .token(token)
            .call()?;

        let result = response.json()?;

        Ok(result)
    }
}

impl<'a, S: State> KeyProvisioningRequestBuilder<'a, Async, S>
where
    S::Name: IsSet,
    S::Disabled: IsUnset,
    S::Offset: IsUnset,
{
    pub async fn create(self) -> Result<CreateKeyResult> {
        let mut handler = self.pool.get().await.expect("Has handler");

        let token = self.provisioning_key.clone();
        let body = self.build();

        let response = handler
            .execute()
            .segments(&[KEY_PROVISION_PATH])
            .method("POST")
            .payload(body)
            .token(token)
            .call()
            .await?;

        let result = response.json().await?;

        Ok(result)
    }
}

impl<'a, S: State> KeyProvisioningRequestBuilder<'a, Sync, S> {
    pub fn list(self) -> Result<ListKeysResult> {
        let mut handler = self.pool.get().expect("Has handler");

        let token = self.provisioning_key.clone();
        let body = self.build();

        let response = handler
            .execute()
            .segments(&[KEY_PROVISION_PATH])
            .method("GET")
            .payload(body)
            .token(token)
            .call()?;

        let result = response.json()?;

        Ok(result)
    }
}

impl<'a, S: State> KeyProvisioningRequestBuilder<'a, Sync, S>
where
    S::Hash: IsSet,
    S::Name: IsUnset,
    S::Disabled: IsUnset,
    S::Offset: IsUnset,
    S::IncludeByokInLimit: IsUnset,
{
    pub fn delete(self) -> Result<DeleteKeyResult> {
        let mut handler = self.pool.get().expect("Has handler");
        let token = self.provisioning_key.clone();
        let mut body = self.build();
        let hash = body.hash.take().unwrap();

        let response = handler
            .execute()
            .segments(&[KEY_PROVISION_PATH, &hash])
            .method("DELETE")
            .payload(body)
            .token(token)
            .call()?;

        let result = response.json()?;

        Ok(result)
    }
}
