use crate::{
    Error, Result,
    constants::COMPLETION_PATH,
    models::{Async, AsyncExecutor, CompletionRequest, Executor, Handler, Mode, Sync},
};

#[derive(Debug)]
pub struct CompletionHandler<M: Mode>(M);

impl<M: Mode> Handler<M> for CompletionHandler<M> {
    const PATH: &str = COMPLETION_PATH;
    type Input = CompletionRequest<M>;
    type Response = M::Response;

    fn new(builder: M::Builder) -> Self {
        Self(M::new(builder))
    }
}

impl Executor for CompletionHandler<Sync> {
    fn execute(self, body: Self::Input) -> Result<reqwest::blocking::Response> {
        let response = self.0.0.json(&body).send().map_err(Error::http)?;

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
        let response = self.0.0.json(&body).send().await.map_err(Error::http)?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let err = response.text().await.map_err(Error::http)?;
            Err(Error::openrouter(err))
        }
    }
}
