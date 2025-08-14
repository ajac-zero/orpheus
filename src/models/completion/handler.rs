use crate::{
    Error, Result,
    constants::COMPLETION_PATH,
    models::{
        common::{
            handler::{AsyncHandler, Handler},
            mode::{Async, Mode, Sync},
        },
        completion::CompletionRequest,
    },
};

#[derive(Debug)]
pub struct CompletionHandler<M: Mode> {
    builder: M,
}

impl Handler for CompletionHandler<Sync> {
    const PATH: &str = COMPLETION_PATH;
    type Input = CompletionRequest<Sync>;

    fn new(builder: reqwest::blocking::RequestBuilder) -> Self {
        Self {
            builder: Sync(builder),
        }
    }

    fn execute(self, body: Self::Input) -> Result<reqwest::blocking::Response> {
        let response = self.builder.0.json(&body).send().map_err(Error::http)?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let err = response.text().map_err(Error::http)?;
            Err(Error::openrouter(err))
        }
    }
}

impl AsyncHandler for CompletionHandler<Async> {
    const PATH: &str = COMPLETION_PATH;
    type Input = CompletionRequest<Async>;

    fn new(builder: reqwest::RequestBuilder) -> Self {
        Self {
            builder: Async(builder),
        }
    }

    async fn execute(self, body: Self::Input) -> Result<reqwest::Response> {
        let response = self
            .builder
            .0
            .json(&body)
            .send()
            .await
            .map_err(Error::http)?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let err = response.text().await.map_err(Error::http)?;
            Err(Error::openrouter(err))
        }
    }
}
