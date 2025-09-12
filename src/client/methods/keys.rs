use crate::{
    client::core::{Mode, OrpheusCore},
    models::keys::{KeyProvisioningRequest, KeyProvisioningRequestBuilder},
};

impl<M: Mode> OrpheusCore<M> {
    /// Initialize a builder for a key provisioning request
    pub fn keys(&self) -> KeyProvisioningRequestBuilder<M> {
        let handler = self.create_handler();
        KeyProvisioningRequest::builder(Some(handler))
    }
}
