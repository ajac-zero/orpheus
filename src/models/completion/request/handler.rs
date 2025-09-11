use crate::{
    Error, Result,
    client::core::{Async, AsyncExecutor, Executor, Handler, Mode, Sync},
    constants::COMPLETION_PATH,
    models::completion::CompletionRequest,
};

#[derive(Debug)]
pub struct CompletionHandler<M: Mode> {
    url: url::Url,
    client: M::Client,
    auth: Option<String>,
}

impl<M: Mode> Handler<M> for CompletionHandler<M> {
    const PATH: &str = COMPLETION_PATH;
    type Input = CompletionRequest<M>;
    type Response = M::Response;

    fn from(core: &crate::client::OrpheusCore<M>) -> Self {
        let url = core.base_url.join(Self::PATH).expect("failed to join url");
        let client = core.client.clone();
        let auth = core.api_key.clone();

        Self { url, client, auth }
    }
}

impl Executor for CompletionHandler<Sync> {
    fn execute(self, body: Self::Input) -> Result<reqwest::blocking::Response> {
        let mut builder = self.client.post(self.url).json(&body);

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

impl AsyncExecutor for CompletionHandler<Async> {
    async fn execute(self, body: Self::Input) -> Result<reqwest::Response> {
        let mut builder = self.client.post(self.url).json(&body);

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
