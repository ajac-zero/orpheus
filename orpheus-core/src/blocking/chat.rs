use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
};

use anyhow::Context;
use either::Either;
use pyo3::prelude::*;
use serde_json::Value;

use super::OrpheusCore;
use crate::{
    constants::CHAT_COMPLETION_PATH,
    models::chat::{ChatCompletion, ChunkStream, message::Messages, prompt::ChatPrompt},
};

pub type CompletionResponse = Either<ChatCompletion, ChunkStream>;

impl OrpheusCore {
    fn chat_completion(
        &self,
        prompt: &ChatPrompt,
        extra_headers: Option<HashMap<String, String>>,
        extra_query: Option<HashMap<String, String>>,
    ) -> Result<CompletionResponse, reqwest::Error> {
        let response = self
            .api_request(CHAT_COMPLETION_PATH, prompt, extra_headers, extra_query)?
            .error_for_status()?;

        let completion = if prompt.is_stream() {
            let buffer = BufReader::new(response).lines();
            let stream = ChunkStream::new(buffer);

            Either::Right(stream)
        } else {
            let completion = response.json::<ChatCompletion>()?;

            Either::Left(completion)
        };

        Ok(completion)
    }
}

#[pymethods]
impl OrpheusCore {
    #[pyo3(signature = (model, messages, stream=None, extra_headers=None, extra_query=None, extra=None))]
    fn native_chat_completions_create(
        &self,
        model: String,
        messages: Messages,
        stream: Option<bool>,
        extra_headers: Option<HashMap<String, String>>,
        extra_query: Option<HashMap<String, String>>,
        extra: Option<&[u8]>,
    ) -> PyResult<CompletionResponse> {
        let extra = extra
            .map(serde_json::from_slice::<Value>)
            .transpose()
            .with_context(|| "Failed to deserialize extra bytes")?;

        let prompt = ChatPrompt::new(model, &messages, stream, extra);

        let completion = self
            .chat_completion(&prompt, extra_headers, extra_query)
            .with_context(|| "Failed to generate chat completion")?;

        Ok(completion)
    }
}
