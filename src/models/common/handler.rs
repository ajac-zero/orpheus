use crate::{
    Result,
    models::common::mode::{Async, Mode, Sync},
};

pub trait Handler<M: Mode> {
    const PATH: &str;
    type Input: serde::Serialize;
    type Response;

    fn new(builder: M::Builder) -> Self;
}

pub trait Executor: Handler<Sync> {
    fn execute(self, body: Self::Input) -> Result<Self::Response>;
}

#[allow(async_fn_in_trait)]
pub trait AsyncExecutor: Handler<Async> {
    async fn execute(self, body: Self::Input) -> Result<Self::Response>;
}
