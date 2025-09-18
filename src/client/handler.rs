mod response;

use bon::bon;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::client::conn::http2;
pub use response::Response;
use tokio::task::JoinHandle;

use crate::{
    Error, Result,
    client::mode::{Async, Sync},
};

#[derive(Debug)]
pub struct Handler<M> {
    pub url: url::Url,
    pub sender: http2::SendRequest<Full<Bytes>>,
    pub _handle: JoinHandle<()>,
    pub mode: M,
}

impl<M> Handler<M> {
    async fn build_request_and_send(
        &mut self,
        segments: &[&str],
        method: &str,
        content_type: &str,
        payload: impl serde::Serialize,
        maybe_token: Option<&str>,
        mode: M,
    ) -> Result<Response<M>> {
        let mut request_uri = self.url.clone();
        request_uri.path_segments_mut().unwrap().extend(segments);

        let body = serde_json::to_vec(&payload)?;

        let builder = hyper::Request::builder()
            .uri(request_uri.as_str())
            .method(method)
            .header(hyper::header::CONTENT_TYPE, content_type);

        let builder = if let Some(token) = maybe_token {
            builder.header(hyper::header::AUTHORIZATION, format!("Bearer {}", token))
        } else {
            builder
        };

        let req = builder.body(Full::<Bytes>::from(body))?;

        let response = self.sender.send_request(req).await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body_bytes = response.collect().await?.to_bytes();
            let error_message = String::from_utf8_lossy(&body_bytes);

            return Err(Error::parse_openrouter_error(status, &error_message));
        }

        Ok(Response::new(response, mode))
    }
}

#[bon]
impl Handler<Sync> {
    #[builder]
    pub fn execute(
        &mut self,
        segments: &[&str],
        #[builder(default = "POST")] method: &str,
        #[builder(default = "application/json")] content_type: &str,
        payload: impl serde::Serialize,
        token: Option<&str>,
    ) -> Result<Response<Sync>> {
        let mode = self.mode.clone();
        let rt = mode.rt.clone();
        rt.block_on(self.build_request_and_send(
            segments,
            method,
            content_type,
            payload,
            token,
            mode,
        ))
    }
}

#[bon]
impl Handler<Async> {
    #[builder(builder_type = AsyncHandlerExecuteBuilder)]
    pub async fn execute(
        &mut self,
        segments: &[&str],
        #[builder(default = "POST")] method: &str,
        #[builder(default = "application/json")] content_type: &str,
        payload: impl serde::Serialize,
        token: Option<&str>,
    ) -> Result<Response<Async>> {
        self.build_request_and_send(segments, method, content_type, payload, token, self.mode)
            .await
    }
}
