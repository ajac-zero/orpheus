use crate::Result;

pub trait Handler {
    const PATH: &str;
    type Input: serde::Serialize;

    fn new(builder: reqwest::blocking::RequestBuilder) -> Self;

    fn execute(self, body: Self::Input) -> Result<reqwest::blocking::Response>;
}

#[allow(async_fn_in_trait)]
pub trait AsyncHandler {
    const PATH: &str;
    type Input: serde::Serialize;

    fn new(builder: reqwest::RequestBuilder) -> Self;

    async fn execute(self, body: Self::Input) -> Result<reqwest::Response>;
}
