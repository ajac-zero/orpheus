use crate::{
    client::{Orpheus, core::Sync},
    models::keys::{KeyProvisioningRequest, KeyProvisioningRequestBuilder},
};

impl Orpheus {
    /// Initialize a builder for a key provisioning request
    pub fn keys(&self) -> KeyProvisioningRequestBuilder<Sync> {
        let handler = self.create_handler();
        KeyProvisioningRequest::builder(Some(handler))
    }
}
