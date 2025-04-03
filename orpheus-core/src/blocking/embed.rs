use super::SyncRest;
use crate::{
    constants::EMBEDDINGS_PATH,
    models::embed::{EmbeddingPrompt, EmbeddingResponse},
    types::ExtrasMap,
};

pub trait SyncEmbed: SyncRest {
    fn embeddings(
        &self,
        prompt: EmbeddingPrompt,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
    ) -> Result<EmbeddingResponse, reqwest::Error> {
        let response = self
            .api_request(EMBEDDINGS_PATH, &prompt, extra_headers, extra_query)?
            .error_for_status()?;

        let completion = response.json::<EmbeddingResponse>()?;

        Ok(completion)
    }
}
