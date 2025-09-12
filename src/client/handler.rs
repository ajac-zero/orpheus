use crate::{
    Result,
    client::{Async, Mode, OrpheusCore, Sync},
};

pub trait Handler<M: Mode> {
    const PATH: &str;
    type Input: serde::Serialize;
    type Response;

    fn from(core: &OrpheusCore<M>) -> Self;
}

pub trait Executor: Handler<Sync> {
    fn execute(self, body: Self::Input) -> Result<Self::Response>;
}

#[allow(async_fn_in_trait)]
pub trait AsyncExecutor: Handler<Async> {
    async fn execute(self, body: Self::Input) -> Result<Self::Response>;
}
