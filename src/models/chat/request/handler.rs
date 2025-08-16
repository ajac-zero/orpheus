use crate::{
    Error, Result,
    constants::CHAT_COMPLETION_PATH,
    models::common::{
        handler::{AsyncExecutor, Executor, Handler},
        mode::{Async, Mode, Sync},
    },
};

#[derive(Debug)]
pub struct ChatHandler<M: Mode>(M);

impl<M: Mode> Handler<M> for ChatHandler<M> {
    const PATH: &str = CHAT_COMPLETION_PATH;
    type Input = super::ChatRequest<M>;
    type Response = M::Response;

    fn new(builder: M::Builder) -> Self {
        Self(M::new(builder))
    }
}

impl Executor for ChatHandler<Sync> {
    fn execute(self, body: Self::Input) -> Result<Self::Response> {
        #[cfg(feature = "otel")]
        super::otel::record_input(&body);

        let response = self.0.0.json(&body).send().map_err(Error::http)?;

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
        super::otel::record_input(&body);

        let response = self.0.0.json(&body).send().await.map_err(Error::http)?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let err = response.text().await.map_err(Error::http)?;
            Err(Error::openrouter(err))
        }
    }
}
