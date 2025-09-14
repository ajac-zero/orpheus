use bytes::Buf;
use http_body_util::BodyExt;

use crate::{
    Result,
    client::mode::{Async, Sync},
};

type HyperResponse = hyper::Response<hyper::body::Incoming>;

pub struct Response<M> {
    pub inner: HyperResponse,
    mode: M,
}

impl<M> Response<M> {
    pub fn new(response: HyperResponse, mode: M) -> Self {
        Self {
            inner: response,
            mode,
        }
    }

    async fn aggregate_and_deserialize<T: serde::de::DeserializeOwned>(
        response: HyperResponse,
    ) -> Result<T> {
        let body = response.collect().await.unwrap().aggregate();
        let value = serde_json::from_reader(body.reader())?;
        Ok(value)
    }
}

impl Response<Async> {
    pub async fn json<T: serde::de::DeserializeOwned>(self) -> Result<T> {
        Self::aggregate_and_deserialize::<T>(self.inner).await
    }
}

impl Response<Sync> {
    pub fn json<T: serde::de::DeserializeOwned>(self) -> Result<T> {
        let rt = self.mode.rt.clone();
        rt.block_on(Self::aggregate_and_deserialize::<T>(self.inner))
    }
}
