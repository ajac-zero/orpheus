use anyhow::Context;
use pyo3::prelude::*;

use super::AsyncOrpheusCore;
use crate::{
    constants::EMBEDDINGS_PATH,
    models::embed::{EmbeddingInput, EmbeddingPrompt, EmbeddingResponse},
    types::ExtrasMap,
};

impl AsyncOrpheusCore {
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

#[pymethods]
impl AsyncOrpheusCore {
    #[pyo3(signature = (input, model, dimensions=None, encoding_format=None, user=None, extra_headers=None, extra_query=None))]
    async fn native_embeddings_create(
        &self,
        input: EmbeddingInput,
        model: String,
        dimensions: Option<i32>,
        encoding_format: Option<String>,
        user: Option<String>,
        extra_headers: ExtrasMap,
        extra_query: ExtrasMap,
    ) -> PyResult<EmbeddingResponse> {
        let prompt = EmbeddingPrompt::new(input, model, encoding_format, dimensions, user);

        let completion = self
            .embeddings(prompt, extra_headers, extra_query)
            .await
            .with_context(|| "Failed to generate embeddings")?;

        Ok(completion)
    }
}
