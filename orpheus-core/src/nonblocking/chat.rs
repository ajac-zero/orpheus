use either::Either;

use super::AsyncRest;
use crate::{
    constants::CHAT_COMPLETION_PATH,
    models::chat::{AsyncChunkStream, ChatCompletion, prompt::ChatPrompt},
    types::ExtrasMap,
};

pub type CompletionResponse = Either<ChatCompletion, AsyncChunkStream>;

pub trait AsyncChat: AsyncRest {
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
