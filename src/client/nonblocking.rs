use crate::constants::*;
use crate::exceptions::OrpheusError;
use crate::models::chat::{AsyncChatResponse, AsyncStream, ChatRequest};
use crate::models::completion::{CompletionRequest, CompletionResponse};
use either::Either::{Left, Right};
use reqwest::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};

#[derive(Debug)]
pub struct Orpheus {
    client: Client,
    api_key: String,
    base_url: String,
}

impl Orpheus {
    /// Create a new Orpheus client with default settings
    pub fn new(api_key: impl Into<String>) -> Self {
        let client = Client::builder()
            .user_agent("Orpheus 1.0")
            .use_rustls_tls()
            .build()
            .unwrap();
        let api_key = api_key.into();
        let base_url = BASE_URL_ENV_VAR.into();

        Self {
            client,
            api_key,
            base_url,
        }
    }

    /// Set the base URL for the API
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Send a chat completion request
    pub async fn chat(&self, request: ChatRequest) -> Result<AsyncChatResponse, OrpheusError> {
        let url = [self.base_url.as_str(), CHAT_COMPLETION_PATH].concat();

        let response = self
            .client
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .bearer_auth(self.api_key.clone())
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(OrpheusError::ApiError {
                status: status.as_u16(),
                message: error_text,
            });
        }

        let chat_response = if request.stream.is_some_and(|x| x) {
            Right(AsyncStream::new(response))
        } else {
            Left(response.json().await?)
        };

        Ok(chat_response)
    }

    /// Send a text completion request
    pub async fn completion(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, OrpheusError> {
        let url = [self.base_url.as_str(), COMPLETION_PATH].concat();

        let response = self
            .client
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .bearer_auth(self.api_key.clone())
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(OrpheusError::ApiError {
                status: status.as_u16(),
                message: error_text,
            });
        }

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
}

#[cfg(test)]
mod tests {
    use std::{env, ops::DerefMut};

    use tokio::io::AsyncBufReadExt;

    use super::*;
    use crate::models::chat::{ChatMessage, ChatStreamChunk, Content, MessageRole};

    #[test]
    fn test_client_creation() {
        let client = Orpheus::new("test_key");
        assert_eq!(client.base_url, "https://openrouter.ai/api/v1");
        assert_eq!(client.api_key, "test_key");
    }

    #[test]
    fn test_client_with_base_url() {
        let client = Orpheus::new("test_key").with_base_url("https://custom-api.example.com/v1");
        assert_eq!(client.base_url, "https://custom-api.example.com/v1");
    }

    #[tokio::test]
    async fn test_chat_request() {
        let api_key = env::var(API_KEY_ENV_VAR).expect("load env var");

        let client = Orpheus::new(api_key);

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

        let client = Orpheus::new(api_key);

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

        let chat_response = response.unwrap().unwrap_right();

        let mut accumulated_content = String::new();
        let mut is_finished = false;

        let mut lines = chat_response.0.lines();

        while let Some(line) = lines.next_line().await.unwrap() {
            if line.is_empty() || line.starts_with(":") {
                continue;
            }

            assert!(line.starts_with("data: "), "Invalid SSE line: {}", line);

            let json_str = &line[6..]; // Remove "data: " prefix and trailing whitespace

            if json_str == "[DONE]" {
                break;
            }

            println!("{:?}", json_str);
            let chunk = serde_json::from_str::<ChatStreamChunk>(json_str).unwrap();

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

        assert!(is_finished);
        println!(
            "Successfully processed streaming chat completion: '{}'",
            accumulated_content
        );
    }

    #[tokio::test]
    async fn test_completion_request() {
        let api_key = env::var(API_KEY_ENV_VAR).expect("load env var");

        let client = Orpheus::new(api_key);

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
