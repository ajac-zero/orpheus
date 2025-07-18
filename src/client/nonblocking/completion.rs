use std::collections::HashMap;

use super::main::AsyncOrpheus;
use crate::{
    Error, Result,
    constants::*,
    models::{
        common::{provider::*, reasoning::*, usage::*},
        completion::{CompletionRequest, CompletionResponse},
    },
};

#[bon::bon]
impl AsyncOrpheus {
    /// Send a text completion request
    #[builder(finish_fn = send, on(String, into))]
    pub async fn completion(
        &self,
        model: String,
        prompt: String,
        models: Option<Vec<String>>,
        provider: Option<ProviderPreferences>,
        reasoning: Option<ReasoningConfig>,
        usage: Option<UsageConfig>,
        transforms: Option<Vec<String>>,
        stream: Option<bool>,
        max_tokens: Option<i32>,
        temperature: Option<f64>,
        seed: Option<i32>,
        top_p: Option<f64>,
        top_k: Option<i32>,
        frequency_penalty: Option<f64>,
        presence_penalty: Option<f64>,
        repetition_penalty: Option<f64>,
        logit_bias: Option<HashMap<String, f64>>,
        top_logprobs: Option<i32>,
        min_p: Option<f64>,
        top_a: Option<f64>,
        user: Option<String>,
    ) -> Result<CompletionResponse> {
        let body = CompletionRequest::new(
            model,
            prompt,
            models,
            provider,
            reasoning,
            usage,
            transforms,
            stream,
            max_tokens,
            temperature,
            seed,
            top_p,
            top_k,
            frequency_penalty,
            presence_penalty,
            repetition_penalty,
            logit_bias,
            top_logprobs,
            min_p,
            top_a,
            user,
        );

        let response = self.execute(COMPLETION_PATH, body).await?;

        let completion_response: CompletionResponse = response.json().await.map_err(Error::http)?;
        Ok(completion_response)
    }
}
