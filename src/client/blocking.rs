use crate::models::chat::{ChatRequest, ChatResponse};
use crate::models::completion::{CompletionRequest, CompletionResponse};
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};

#[derive(Debug)]
pub struct Orpheus {
    client: Client,
    api_key: String,
    base_url: String,
}

#[derive(Debug)]
pub enum OrpheusError {
    Http(reqwest::Error),
    Serialization(serde_json::Error),
    ApiError { status: u16, message: String },
    MissingApiKey,
}

impl From<reqwest::Error> for OrpheusError {
    fn from(err: reqwest::Error) -> Self {
        OrpheusError::Http(err)
    }
}

impl From<serde_json::Error> for OrpheusError {
    fn from(err: serde_json::Error) -> Self {
        OrpheusError::Serialization(err)
    }
}

impl std::fmt::Display for OrpheusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrpheusError::Http(e) => write!(f, "HTTP error: {}", e),
            OrpheusError::Serialization(e) => write!(f, "Serialization error: {}", e),
            OrpheusError::ApiError { status, message } => {
                write!(f, "API error {}: {}", status, message)
            }
            OrpheusError::MissingApiKey => write!(f, "API key is required"),
        }
    }
}

impl std::error::Error for OrpheusError {}

impl Orpheus {
    /// Create a new Orpheus client with default settings
    pub fn new(api_key: impl Into<String>) -> Self {
        let client = Client::builder()
            .user_agent("Orpheus 1.0")
            .use_rustls_tls()
            .build()
            .unwrap();
        let api_key = api_key.into();
        let base_url = super::OPENROUTER_BASE_URL.into();

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

    /// Build headers for requests
    fn build_headers(&self) -> Result<HeaderMap, OrpheusError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let auth_value = HeaderValue::from_str(&format!("Bearer {}", self.api_key))
            .map_err(|_| OrpheusError::MissingApiKey)?;
        headers.insert(AUTHORIZATION, auth_value);

        Ok(headers)
    }

    /// Send a chat completion request
    pub fn chat(&self, request: ChatRequest) -> Result<ChatResponse, OrpheusError> {
        let headers = self.build_headers()?;
        let url = format!("{}/chat/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(OrpheusError::ApiError {
                status: status.as_u16(),
                message: error_text,
            });
        }

        let chat_response: ChatResponse = response.json()?;
        Ok(chat_response)
    }

    /// Send a text completion request
    pub fn completion(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, OrpheusError> {
        let headers = self.build_headers()?;
        let url = format!("{}/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(OrpheusError::ApiError {
                status: status.as_u16(),
                message: error_text,
            });
        }

        let completion_response: CompletionResponse = response.json()?;
        Ok(completion_response)
    }

    /// Convenience method for simple chat requests
    pub fn simple_chat(
        &self,
        model: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<ChatResponse, OrpheusError> {
        let request = ChatRequest::simple(model, message);
        self.chat(request)
    }

    /// Convenience method for chat with system prompt
    pub fn chat_with_system(
        &self,
        model: impl Into<String>,
        system_prompt: impl Into<String>,
        user_message: impl Into<String>,
    ) -> Result<ChatResponse, OrpheusError> {
        let request = ChatRequest::with_system(model, system_prompt, user_message);
        self.chat(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::chat::{ChatMessage, Content, MessageRole};

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

    #[test]
    fn test_headers() {
        let client = Orpheus::new("test_key");
        let headers = client.build_headers().unwrap();

        assert!(headers.contains_key(CONTENT_TYPE));
        assert!(headers.contains_key(AUTHORIZATION));

        let auth_header = headers.get(AUTHORIZATION).unwrap().to_str().unwrap();
        assert_eq!(auth_header, "Bearer test_key");
    }

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatRequest::simple("gpt-3.5-turbo", "Hello world");

        // Test that we can serialize the request (this would normally be sent to the API)
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("gpt-3.5-turbo"));
        assert!(json.contains("Hello world"));
    }

    #[test]
    fn test_completion_request_serialization() {
        let request = CompletionRequest {
            model: "gpt-3.5-turbo".to_string(),
            prompt: "Complete this sentence:".to_string(),
            models: None,
            provider: None,
            reasoning: None,
            usage: None,
            transforms: None,
            stream: Some(false),
            max_tokens: Some(50),
            temperature: Some(0.7),
            seed: None,
            top_p: None,
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
            repetition_penalty: None,
            logit_bias: None,
            top_logprobs: None,
            min_p: None,
            top_a: None,
            user: None,
        };

        // Test that we can serialize the request
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Complete this sentence:"));
    }

    #[test]
    fn test_chat_request() {
        let client = Orpheus::new(
            "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",
        );

        let response = client.chat(ChatRequest::new(
            "deepseek/deepseek-r1-0528-qwen3-8b:free",
            vec![
                ChatMessage::new_system(Content::simple("You are a friend")),
                ChatMessage::new_user(Content::simple("Hello!")),
            ],
        ));
        println!("{:?}", response);

        assert!(response.is_ok());

        let chat_response = response.unwrap();
        assert!(chat_response.id.is_some());
        assert!(chat_response.choices.is_some());

        let choices = chat_response.choices.unwrap();
        assert!(!choices.is_empty());
    }
}
