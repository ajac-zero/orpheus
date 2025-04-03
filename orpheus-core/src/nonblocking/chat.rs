use anyhow::Context;
use either::Either;
use pyo3::prelude::*;
use serde_json::Value;

use super::AsyncOrpheusCore;
use crate::{
    constants::CHAT_COMPLETION_PATH,
    models::chat::{AsyncChunkStream, ChatCompletion, message::Messages, prompt::ChatPrompt},
    types::ExtrasMap,
};

pub type CompletionResponse = Either<ChatCompletion, AsyncChunkStream>;

impl AsyncOrpheusCore {
    async fn chat_completion(
        &self,
        prompt: &ChatPrompt<'_>,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
    ) -> Result<CompletionResponse, reqwest::Error> {
        let response = self
            .api_request(CHAT_COMPLETION_PATH, prompt, extra_headers, extra_query)
            .await?
            .error_for_status()?;

        let completion = if prompt.is_stream() {
            let bytes_steam = Box::pin(response.bytes_stream());
            let stream = AsyncChunkStream::new(bytes_steam);

            Either::Right(stream)
        } else {
            let completion = response.json::<ChatCompletion>().await?;

            Either::Left(completion)
        };

        Ok(completion)
    }
}

#[pymethods]
impl AsyncOrpheusCore {
    #[pyo3(signature = (model, messages, stream=None, extra_headers=None, extra_query=None, extra=None))]
    async fn native_chat_completions_create(
        &self,
        model: String,
        messages: Messages,
        stream: Option<bool>,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
        extra: Option<Vec<u8>>,
    ) -> PyResult<CompletionResponse> {
        let extra = extra
            .as_deref()
            .map(serde_json::from_slice::<Value>)
            .transpose()
            .expect("Serialize bytes to json");

        let prompt = ChatPrompt::new(model, &messages, stream, extra);

        let completion = self
            .chat_completion(&prompt, extra_headers, extra_query)
            .await
            .with_context(|| "Failed to generate chat completion")?;

        Ok(completion)
    }
}
