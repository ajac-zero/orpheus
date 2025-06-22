use crate::constants::*;
use crate::models::chat::{AsyncChatResponse, ChatMessage, ChatRequest, Content};
use crate::models::completion::{CompletionRequest, CompletionResponse};
use either::Either::{Left, Right};
use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use url::Url;

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
    pub fn with_base_url(mut self, base_url: impl Into<Url>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Set the base URL for the API
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Send a chat completion request
    pub fn chat(&self) -> Result<ChatRequestBuilder, anyhow::Error> {
        let url = self
            .base_url
            .join(CHAT_COMPLETION_PATH)
            .expect("Is valid Url");
        let token = self.api_key.clone();
        let client = self.client.clone();
        let request = ChatRequest::default();

        let req = ChatRequestBuilder {
            url,
            token,
            client,
            request,
        }
        .stream(false);

        Ok(req)
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
    ) -> Result<AsyncChatResponse, anyhow::Error> {
        let message = ChatMessage::user(Content::simple(message));

        self.chat()?
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
    ) -> Result<AsyncChatResponse, anyhow::Error> {
        let messages = vec![
            ChatMessage::system(Content::simple(system_prompt)),
            ChatMessage::user(Content::simple(user_message)),
        ];

        self.chat()?.model(model).messages(messages).send().await
    }

    /// Convenience method for simple streaming requests
    pub async fn simple_chat_stream(
        &self,
        model: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<AsyncChatResponse, anyhow::Error> {
        let message = ChatMessage::user(Content::simple(message));

        self.chat()?
            .model(model)
            .messages(vec![message])
            .stream(true)
            .send()
            .await
    }
}

struct ChatRequestBuilder {
    url: Url,
    token: Option<String>,
    client: Client,
    request: ChatRequest,
}

impl ChatRequestBuilder {
    pub async fn send(self) -> Result<AsyncChatResponse, anyhow::Error> {
        let response = self
            .client
            .post(self.url.as_str())
            .header(CONTENT_TYPE, "application/json")
            .bearer_auth(self.token.unwrap_or_else(String::new))
            .json(&self.request)
            .send()
            .await?
            .error_for_status()?;

        let chat_response = if self.request.stream.is_some_and(|x| x) {
            Right(response.into())
        } else {
            Left(response.json().await?)
        };

        Ok(chat_response)
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.request.model = model.into();
        self
    }

    pub fn messages(mut self, messages: Vec<ChatMessage>) -> Self {
        self.request.messages = messages;
        self
    }
    //     pub models: Option<Vec<String>>,
    //     pub provider: Option<ProviderPreferences>,
    //     pub reasoning: Option<ReasoningConfig>,
    //     pub usage: Option<UsageConfig>,
    //     pub transforms: Option<Vec<String>>,
    //     pub stream: Option<bool>,
    pub fn stream(mut self, stream: bool) -> Self {
        self.request.stream = Some(stream);
        self
    }
    //     pub max_tokens: Option<i32>,
    //     pub temperature: Option<f64>,
    //     pub seed: Option<i32>,
    //     pub top_p: Option<f64>,
    //     pub frequency_penalty: Option<f64>,
    //     pub presence_penalty: Option<f64>,
    //     pub repetition_penalty: Option<f64>,
    //     pub logit_bias: Option<HashMap<String, f64>>,
    //     pub top_logprobs: Option<i32>,
    //     pub min_p: Option<f64>,
    //     pub top_a: Option<f64>,
    //     pub user: Option<String>,
    // }
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

        let response = client
            .chat()
            .expect("client has correct credentials")
            .model("deepseek/deepseek-r1-0528-qwen3-8b:free")
            .messages(vec![
                ChatMessage::system(Content::simple("You are a friend")),
                ChatMessage::user(Content::simple("Hello!")),
            ])
            .send()
            .await;
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
