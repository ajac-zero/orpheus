use std::collections::HashMap;

use reqwest::{Client, header::CONTENT_TYPE};
use url::Url;

use crate::{
    constants::*,
    models::{
        chat::*,
        completion::{self, CompletionRequest, CompletionResponse},
    },
};

#[derive(Debug, Clone)]
pub struct AsyncOrpheus {
    client: Client,
    api_key: Option<String>,
    base_url: Url,
}

impl Default for AsyncOrpheus {
    fn default() -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT_NAME)
            .use_rustls_tls()
            .build()
            .expect("build request client");

        Self {
            client,
            api_key: None,
            base_url: Url::parse(DEFAULT_BASE_URL).expect("Default is valid Url"),
        }
    }
}

impl AsyncOrpheus {
    /// Create a new Orpheus client with provided key and default base url
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::default().with_api_key(api_key)
    }

    /// Set the base URL for the API
    pub fn with_base_url<U>(mut self, base_url: U) -> crate::Result<Self>
    where
        U: TryInto<Url>,
        U::Error: Into<crate::Error>,
    {
        self.base_url = base_url.try_into().map_err(Into::into)?;
        Ok(self)
    }

    /// Set the base URL for the API
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub async fn execute(
        &self,
        path: &str,
        body: impl serde::Serialize,
    ) -> crate::Result<reqwest::Response> {
        let url = self.base_url.join(path)?;
        let token = self
            .api_key
            .as_ref()
            .map_or_else(String::new, |key| key.clone());
        let response = self
            .client
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .bearer_auth(token)
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let err = response.text().await?;
            Err(crate::Error::OpenRouter(err))
        }
    }

    /// Convenience method for simple chat requests
    pub async fn simple_chat(
        &self,
        model: impl Into<String>,
        message: impl Into<String>,
    ) -> crate::Result<ChatCompletion> {
        let message = ChatMessage::user(Content::simple(message));

        self.chat()
            .model(model)
            .messages(vec![message])
            .send()
            .await
    }

    /// Convenience method for chat with system prompt
    pub async fn chat_with_system(
        &self,
        model: impl Into<String>,
        system_prompt: impl Into<String>,
        user_message: impl Into<String>,
    ) -> crate::Result<ChatCompletion> {
        let messages = vec![
            ChatMessage::system(Content::simple(system_prompt)),
            ChatMessage::user(Content::simple(user_message)),
        ];

        self.chat().model(model).messages(messages).send().await
    }

    /// Convenience method for simple streaming requests
    pub async fn simple_chat_stream(
        &self,
        model: impl Into<String>,
        message: impl Into<String>,
    ) -> crate::Result<AsyncStream> {
        let message = ChatMessage::user(Content::simple(message));

        self.chat_stream()
            .model(model)
            .messages(vec![message])
            .send()
            .await
    }
}

#[bon::bon]
impl AsyncOrpheus {
    #[builder(finish_fn = send, on(String, into))]
    async fn chat(
        &self,
        model: String,
        messages: Vec<ChatMessage>,
        models: Option<Vec<String>>,
        provider: Option<ProviderPreferences>,
        reasoning: Option<ReasoningConfig>,
        usage: Option<UsageConfig>,
        transforms: Option<Vec<String>>,
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
    ) -> crate::Result<ChatCompletion> {
        let stream = Some(false);
        let body = ChatRequest::new(
            model,
            messages,
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

        let response = self.execute(CHAT_COMPLETION_PATH, body).await?;

        let chat_completion = response.json::<ChatCompletion>().await?;

        Ok(chat_completion)
    }

    #[builder(finish_fn = send, on(String, into))]
    async fn chat_stream(
        &self,
        model: String,
        messages: Vec<ChatMessage>,
        models: Option<Vec<String>>,
        provider: Option<ProviderPreferences>,
        reasoning: Option<ReasoningConfig>,
        usage: Option<UsageConfig>,
        transforms: Option<Vec<String>>,
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
    ) -> crate::Result<AsyncStream> {
        let stream = Some(true);
        let body = ChatRequest::new(
            model,
            messages,
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

        let response = self.execute(CHAT_COMPLETION_PATH, body).await?;

        Ok(response.into())
    }

    /// Send a text completion request
    #[builder(finish_fn = send, on(String, into))]
    pub async fn completion(
        &self,
        model: String,
        prompt: String,
        models: Option<Vec<String>>,
        provider: Option<completion::ProviderPreferences>,
        reasoning: Option<completion::ReasoningConfig>,
        usage: Option<completion::UsageConfig>,
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
    ) -> crate::Result<CompletionResponse> {
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

        let completion_response: CompletionResponse = response.json().await?;
        Ok(completion_response)
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use futures_lite::StreamExt;

    use super::*;
    use crate::models::chat::{ChatMessage, Content};

    #[test]
    fn test_client_creation() {
        let client = AsyncOrpheus::new("test_key");
        assert_eq!(
            client.base_url,
            Url::parse("https://openrouter.ai/api/v1/").unwrap()
        );
        assert_eq!(client.api_key, Some("test_key".to_string()));
    }

    #[test]
    fn test_client_with_base_url() {
        let client = AsyncOrpheus::new("test_key")
            .with_base_url("https://custom-api.example.com/v1")
            .unwrap();
        assert_eq!(
            client.base_url,
            Url::parse("https://custom-api.example.com/v1").unwrap()
        );
    }

    #[tokio::test]
    async fn test_chat_request() {
        let api_key = env::var(API_KEY_ENV_VAR).expect("load env var");

        let client = AsyncOrpheus::new(api_key);

        let response = client
            .chat()
            .model("deepseek/deepseek-r1-0528-qwen3-8b:free")
            .messages(vec![
                ChatMessage::system(Content::simple("You are a friend")),
                ChatMessage::user(Content::simple("Hello!")),
            ])
            .send()
            .await;
        println!("{:?}", response);

        assert!(response.is_ok());

        let chat_response = response.unwrap();
        assert!(chat_response.id.is_some());
        assert!(chat_response.choices.is_some());

        let choices = chat_response.choices.unwrap();
        assert!(!choices.is_empty());
    }

    #[tokio::test]
    async fn test_chat_stream_request() {
        let api_key = env::var(API_KEY_ENV_VAR).expect("load env var");

        let client = AsyncOrpheus::new(api_key);

        let response = client
            .chat_stream()
            .model("deepseek/deepseek-r1-0528-qwen3-8b:free")
            .messages(vec![
                ChatMessage::system(Content::simple("You are a friend")),
                ChatMessage::user(Content::simple("Hello!")),
            ])
            .send()
            .await;
        println!("{:?}", response);

        assert!(response.is_ok());

        let mut chat_response = response.unwrap();

        let mut accumulated_content = String::new();
        let mut is_finished = false;

        let mut count = 0;
        while let Some(chunk) = chat_response.next().await {
            println!("{:?}", chunk);
            count = count + 1;
            let chunk = chunk.unwrap();
            assert_eq!(chunk.object, "chat.completion.chunk");
            assert_eq!(chunk.choices.len(), 1);

            let choice = &chunk.choices[0];

            // Accumulate content
            if let Some(content) = &choice.delta.content {
                accumulated_content.push_str(content);
            }

            // Check for completion
            if choice.finish_reason.is_some() {
                is_finished = true;
                assert_eq!(choice.finish_reason, Some("stop".to_string()));
            }
        }

        println!("Processed chunks: {}", count);
        assert!(is_finished);
        println!(
            "Successfully processed streaming chat completion: '{}'",
            accumulated_content
        );
    }

    #[tokio::test]
    async fn test_completion_request() {
        let api_key = env::var(API_KEY_ENV_VAR).expect("load env var");

        let client = AsyncOrpheus::new(api_key);

        let response = client
            .completion()
            .model("openai/gpt-3.5-turbo")
            .prompt("The best city in the world is ")
            .send()
            .await;
        println!("{:?}", response);

        // assert!(response.is_ok());
    }
}
