use std::collections::HashMap;

use anyhow::Context;
use pyo3::prelude::*;

use super::OrpheusCore;
use crate::{
    constants::EMBEDDINGS_PATH,
    models::embed::{EmbeddingInput, EmbeddingPrompt, EmbeddingResponse},
};

impl OrpheusCore {
    fn embeddings(
        &self,
        prompt: EmbeddingPrompt,
        extra_headers: Option<HashMap<String, String>>,
        extra_query: Option<HashMap<String, String>>,
    ) -> Result<EmbeddingResponse, reqwest::Error> {
        let response = self
            .api_request(EMBEDDINGS_PATH, &prompt, extra_headers, extra_query)?
            .error_for_status()?;

        let completion = response.json::<EmbeddingResponse>()?;

        Ok(completion)
    }
}

#[pymethods]
impl OrpheusCore {
    #[pyo3(signature = (input, model, dimensions=None, encoding_format=None, user=None, extra_headers=None, extra_query=None))]
    fn native_embeddings_create(
        &self,
        input: EmbeddingInput,
        model: String,
        dimensions: Option<i32>,
        encoding_format: Option<String>,
        user: Option<String>,
        extra_headers: Option<HashMap<String, String>>,
        extra_query: Option<HashMap<String, String>>,
    ) -> PyResult<EmbeddingResponse> {
        let prompt = EmbeddingPrompt::new(input, model, encoding_format, dimensions, user);

        let completion = self
            .embeddings(prompt, extra_headers, extra_query)
            .with_context(|| "Failed to generate embeddings")?;

        Ok(completion)
    }
}
