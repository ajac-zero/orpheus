#![allow(clippy::large_enum_variant)]

use std::io::{BufRead, BufReader, Lines};

use pyo3::exceptions::{PyStopIteration, PyValueError};
use pyo3::prelude::*;

use reqwest::blocking::Response;

use crate::types::chat::{ChatCompletion, ChatCompletionChunk};
use crate::types::prompt::Prompt;
use crate::types::ExtrasMap;

use super::SyncRest;

const CHAT_COMPLETION_PATH: &str = "/chat/completions";

pub trait SyncChat: SyncRest {
    fn chat_completion(
        &self,
        prompt: &Prompt,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
    ) -> Result<CompletionResponse, reqwest::Error> {
        let response = self
            .api_request(CHAT_COMPLETION_PATH, prompt, extra_headers, extra_query)?
            .error_for_status()?;

        let completion = if prompt.is_stream() {
            let buffer = BufReader::new(response).lines();
            let stream = Stream::new(buffer);

            CompletionResponse::Stream(stream)
        } else {
            let completion = response.json::<ChatCompletion>()?;

            CompletionResponse::Completion(completion)
        };

        Ok(completion)
    }
}

#[derive(IntoPyObject)]
pub enum CompletionResponse {
    #[pyo3(transparent)]
    Completion(ChatCompletion),
    #[pyo3(transparent)]
    Stream(Stream),
}

#[pyclass]
pub struct Stream {
    buffer: Lines<BufReader<Response>>,
}

impl Stream {
    fn new(buffer: Lines<BufReader<Response>>) -> Self {
        Self { buffer }
    }
}

#[pymethods]
impl Stream {
    fn __next__(&mut self) -> PyResult<ChatCompletionChunk> {
        loop {
            match self.buffer.next() {
                Some(chunk) => {
                    let line = chunk.unwrap();

                    if line.is_empty() {
                        continue;
                    }

                    let data = &line[6..];

                    if data == "[DONE]" {
                        break Err(PyStopIteration::new_err("end of stream"));
                    }

                    break serde_json::from_str::<ChatCompletionChunk>(data)
                        .map_err(|e| PyValueError::new_err(format!("{e}")));
                }
                None => break Err(PyStopIteration::new_err("end of stream")),
            }
        }
    }

    fn __iter__(self_: PyRef<Self>) -> PyRef<Self> {
        self_
    }
}
