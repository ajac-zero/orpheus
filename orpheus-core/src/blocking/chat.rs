use std::io::{BufRead, BufReader};

use either::Either;

use super::SyncRest;
use crate::{
    constants::CHAT_COMPLETION_PATH,
    models::chat::{ChatCompletion, ChunkStream, prompt::ChatPrompt},
    types::ExtrasMap,
};

pub type CompletionResponse = Either<ChatCompletion, ChunkStream>;

pub trait SyncChat: SyncRest {
    fn chat_completion(
        &self,
        prompt: &ChatPrompt,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
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
