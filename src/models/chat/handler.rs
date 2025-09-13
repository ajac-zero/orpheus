use reqwest::header::CONTENT_TYPE;
use url::Url;

use crate::{
    Error, Result,
    client::{Async, AsyncExecutor, Executor, Handler, Mode, OrpheusCore, Sync},
    constants::CHAT_COMPLETION_PATH,
    models::chat::ChatRequest,
};

#[derive(Debug)]
pub(crate) struct ChatHandler<M: Mode> {
    url: Url,
    client: M::Client,
    auth: Option<String>,
}

impl<M: Mode> Handler<M> for ChatHandler<M> {
    const PATH: &str = CHAT_COMPLETION_PATH;
    type Input = ChatRequest<M>;
    type Response = M::Response;

    fn from(core: &OrpheusCore<M>) -> Self {
        let url = core.base_url.join(Self::PATH).expect("failed to join url");
        let client = core.client.clone();
        let auth = core.api_key.clone();

        Self { url, client, auth }
    }
}

impl Executor for ChatHandler<Sync> {
    fn execute(self, body: Self::Input) -> Result<Self::Response> {
        #[cfg(feature = "otel")]
        crate::otel::record_input(&body);

        let mut builder = self
            .client
            .post(self.url)
            .header(CONTENT_TYPE, "application/json")
            .json(&body);

        if let Some(token) = self.auth {
            builder = builder.bearer_auth(token);
        }

        let response = builder.send().map_err(Error::http)?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let err = response.text().map_err(Error::http)?;
            Err(Error::openrouter(err))
        }
    }
}

impl AsyncExecutor for ChatHandler<Async> {
    async fn execute(self, body: Self::Input) -> Result<Self::Response> {
        #[cfg(feature = "otel")]
        crate::otel::record_input(&body);

        let mut builder = self
            .client
            .post(self.url)
            .header(CONTENT_TYPE, "application/json")
            .json(&body);

        if let Some(token) = self.auth {
            builder = builder.bearer_auth(token);
        }

        let response = builder.send().await.map_err(Error::http)?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let err = response.text().await.map_err(Error::http)?;
            Err(Error::openrouter(err))
        }
    }
}
