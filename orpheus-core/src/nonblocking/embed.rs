use crate::types::embed::{EmbeddingPrompt, EmbeddingResponse};
use crate::types::ExtrasMap;

use super::AsyncRest;

const EMBEDDINGS_PATH: &str = "/v1/embeddings";

pub trait AsyncEmbed: AsyncRest {
    async fn embeddings(
        &self,
        prompt: EmbeddingPrompt,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
    ) -> Result<EmbeddingResponse, reqwest::Error> {
        let response = self
            .api_request(EMBEDDINGS_PATH, &prompt, extra_headers, extra_query)
            .await?
            .error_for_status()?;

        let completion = response.json::<EmbeddingResponse>().await?;

        Ok(completion)
    }
}
