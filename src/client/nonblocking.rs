use crate::constants::*;
use crate::exceptions::OrpheusError;
use crate::models::chat::{AsyncChatResponse, ChatRequest};
use crate::models::completion::{CompletionRequest, CompletionResponse};
use either::Either::{Left, Right};
use reqwest::Client;
use reqwest::header::CONTENT_TYPE;

#[derive(Debug, Clone)]
pub struct AsyncOrpheus {
    client: Client,
    api_key: Option<String>,
    base_url: Option<String>,
}

impl Default for AsyncOrpheus {
    fn default() -> Self {
        let client = Client::builder()
            .user_agent("Orpheus 1.0")
            .use_rustls_tls()
            .build()
            .expect("build request client");

        Self {
            client,
            api_key: None,
            base_url: None,
        }
    }
}

impl AsyncOrpheus {
    /// Create a new Orpheus client with provided key and default base url
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::default()
            .with_api_key(api_key)
            .with_base_url(DEFAULT_BASE_URL)
    }

    /// Set the base URL for the API
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Set the base URL for the API
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    fn get_base_url(&self) -> Result<String, OrpheusError> {
        self.base_url
            .clone()
            .ok_or(OrpheusError::Anyhow("No base url set".into()))
    }

    fn get_url_path(&self, path: &str) -> Result<String, OrpheusError> {
        self.get_base_url().map(|url| [url.as_str(), path].concat())
    }

    fn get_api_key(&self) -> Result<String, OrpheusError> {
        self.api_key
            .clone()
            .ok_or(OrpheusError::Anyhow("No api key set".into()))
    }

    /// Send a chat completion request
    pub async fn chat(&self, request: ChatRequest) -> Result<AsyncChatResponse, OrpheusError> {
        let url = self.get_url_path(CHAT_COMPLETION_PATH)?;
        let token = self.get_api_key()?;

        let response = self
            .client
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .bearer_auth(token)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        let chat_response = if request.stream.is_some_and(|x| x) {
            Right(response.into())
        } else {
            Left(response.json().await?)
        };

        Ok(chat_response)
    }

    /// Send a text completion request
    pub async fn completion(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, anyhow::Error> {
        let url = self.get_url_path(COMPLETION_PATH)?;
        let token = self.get_api_key()?;

        let response = self
            .client
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .bearer_auth(token)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        let completion_response: CompletionResponse = response.json().await?;
        Ok(completion_response)
    }

    /// Convenience method for simple chat requests
    pub async fn simple_chat(
        &self,
        model: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<AsyncChatResponse, OrpheusError> {
        let request = ChatRequest::simple(model, message);
        self.chat(request).await
    }

    /// Convenience method for chat with system prompt
    pub async fn chat_with_system(
        &self,
        model: impl Into<String>,
        system_prompt: impl Into<String>,
        user_message: impl Into<String>,
    ) -> Result<AsyncChatResponse, OrpheusError> {
        let request = ChatRequest::with_system(model, system_prompt, user_message);
        self.chat(request).await
    }

    /// Convenience method for simple streaming requests
    pub async fn simple_chat_stream(
        &self,
        model: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<AsyncChatResponse, OrpheusError> {
        let request = ChatRequest::simple_stream(model, message);
        self.chat(request).await
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use futures_util::StreamExt;

    use super::*;
    use crate::models::chat::{ChatMessage, Content};

    #[test]
    fn test_client_creation() {
        let client = AsyncOrpheus::new("test_key");
        assert_eq!(
            client.base_url,
            Some("https://openrouter.ai/api/v1".to_string())
        );
        assert_eq!(client.api_key, Some("test_key".to_string()));
    }

    #[test]
    fn test_client_with_base_url() {
        let client =
            AsyncOrpheus::new("test_key").with_base_url("https://custom-api.example.com/v1");
        assert_eq!(
            client.base_url,
            Some("https://custom-api.example.com/v1".to_string())
        );
    }

    #[tokio::test]
    async fn test_chat_request() {
        let api_key = env::var(API_KEY_ENV_VAR).expect("load env var");

        let client = AsyncOrpheus::new(api_key);

        let request = ChatRequest::builder()
            .model("deepseek/deepseek-r1-0528-qwen3-8b:free".into())
            .messages(vec![
                ChatMessage::system(Content::simple("You are a friend")),
                ChatMessage::user(Content::simple("Hello!")),
            ])
            .build();

        let response = client.chat(request).await;
        println!("{:?}", response);

        assert!(response.is_ok());

        let chat_response = response.unwrap().unwrap_left();
        assert!(chat_response.id.is_some());
        assert!(chat_response.choices.is_some());

        let choices = chat_response.choices.unwrap();
        assert!(!choices.is_empty());
    }

    #[tokio::test]
    async fn test_chat_stream_request() {
        let api_key = env::var(API_KEY_ENV_VAR).expect("load env var");

        let client = AsyncOrpheus::new(api_key);

        let request = ChatRequest::builder()
            .model("deepseek/deepseek-r1-0528-qwen3-8b:free".into())
            .messages(vec![
                ChatMessage::system(Content::simple("You are a friend")),
                ChatMessage::user(Content::simple("Hello!")),
            ])
            .stream(true)
            .build();

        let response = client.chat(request).await;
        println!("{:?}", response);

        assert!(response.is_ok());

        let mut chat_response = response.unwrap().unwrap_right();

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

        let request = CompletionRequest::builder()
            .model("openai/gpt-3.5-turbo".into())
            .prompt("The greatest capital in the world is ".into())
            .build();
        let response = client.completion(request).await;
        println!("{:?}", response);

        assert!(response.is_ok());

        let completion_response = response.unwrap();
        assert!(completion_response.id.is_some());
        assert!(completion_response.choices.is_some());

        let choices = completion_response.choices.unwrap();
        assert!(!choices.is_empty());
    }
}
