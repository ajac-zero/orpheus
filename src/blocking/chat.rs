use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
};

use anyhow::Context;
use either::Either;
use pyo3::prelude::*;

use super::OrpheusCore;
use crate::{
    constants::CHAT_COMPLETION_PATH,
    models::chat::{ChatCompletion, ChatPrompt, ChunkStream, Messages},
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
    #[pyo3(signature = (model, messages, stream=None, tools=None, extra_headers=None, extra_query=None, extra=None))]
    fn native_chat_completions_create(
        &self,
        model: String,
        messages: Messages,
        stream: Option<bool>,
        tools: Option<&[u8]>,
        extra_headers: Option<HashMap<String, String>>,
        extra_query: Option<HashMap<String, String>>,
        extra: Option<&[u8]>,
    ) -> PyResult<CompletionResponse> {
        let prompt = ChatPrompt::new(model, &messages, stream, tools, extra)?;

        let completion = self
            .chat_completion(&prompt, extra_headers, extra_query)
            .with_context(|| "Failed to generate chat completion")?;

        Ok(completion)
    }
}
