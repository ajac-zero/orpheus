use crate::{
    Result,
    client::{OrpheusCore, mode::Mode},
    models::keys::{KeyProvisioningRequest, KeyProvisioningRequestBuilder},
};

impl<'a, M: Mode> OrpheusCore<M> {
    /// Initialize a builder for a key provisioning request
    pub fn keys(&'a self) -> Result<KeyProvisioningRequestBuilder<'a, M>> {
        let provisioning_key = self.keystore.get_provisioning_key()?;

        Ok(KeyProvisioningRequest::builder(
            &self.pool,
            provisioning_key,
        ))
    }
}
