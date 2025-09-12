use reqwest::Method;

use crate::{
    Error, Result,
    client::{Async, AsyncExecutor, Executor, Handler, Mode, Sync},
    constants::KEY_PROVISION_PATH,
    models::keys::KeyProvisioningRequest,
};

#[derive(Debug)]
pub(crate) struct ProvisionHandler<M: Mode> {
    url: url::Url,
    client: M::Client,
    provisioning_key: Option<String>,
}

impl<M: Mode> Handler<M> for ProvisionHandler<M> {
    const PATH: &str = KEY_PROVISION_PATH;
    type Input = KeyProvisioningRequest<M>;
    type Response = M::Response;

    fn from(core: &crate::client::OrpheusCore<M>) -> Self {
        let url = core.base_url.join(Self::PATH).expect("failed to join url");
        let client = core.client.clone();
        let provisioning_key = core.provisioning_key.clone();

        Self {
            url,
            client,
            provisioning_key,
        }
    }
}

impl Executor for ProvisionHandler<Sync> {
    fn execute(self, body: Self::Input) -> Result<Self::Response> {
        let token = self.provisioning_key.ok_or(Error::MissingProvisioningKey)?;
        let mut url = self.url.clone();

        if let Some(hash) = body.hash.as_ref() {
            let mut path_segments = url.path_segments_mut().unwrap();
            path_segments.push(&hash);
        }

        let builder = self
            .client
            .request(body.method.clone(), url)
            .bearer_auth(token);

        let builder = match body.method {
            Method::POST | Method::PATCH => builder.json(&body),
            Method::GET | Method::DELETE => {
                if let Some(offset) = body.offset {
                    builder.query(&[("offset", offset)])
                } else {
                    builder
                }
            }
            _ => unimplemented!(),
        };

        let response = builder.send().map_err(Error::http)?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let err = response.text().map_err(Error::http)?;
            Err(Error::openrouter(err))
        }
    }
}

impl AsyncExecutor for ProvisionHandler<Async> {
    async fn execute(self, body: Self::Input) -> Result<Self::Response> {
        let token = self.provisioning_key.ok_or(Error::MissingProvisioningKey)?;
        let mut url = self.url.clone();

        if let Some(hash) = body.hash.as_ref() {
            let mut path_segments = url.path_segments_mut().unwrap();
            path_segments.push(&hash);
        }
        println!("URL: {}", url);

        let builder = self
            .client
            .request(body.method.clone(), url)
            .bearer_auth(token);

        let builder = match body.method {
            Method::POST | Method::PATCH => builder.json(&body),
            Method::GET | Method::DELETE => {
                if let Some(offset) = body.offset {
                    builder.query(&[("offset", offset)])
                } else {
                    builder
                }
            }
            _ => unimplemented!(),
        };

        let response = builder.send().await.map_err(Error::http)?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let err = response.text().await.map_err(Error::http)?;
            Err(Error::openrouter(err))
        }
    }
}
