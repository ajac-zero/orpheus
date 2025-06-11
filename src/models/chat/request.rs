use bon::Builder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct ChatRequest {
    /// The model ID to use. If unspecified, the user's default is used.
    pub model: String,

    /// List of messages in the conversation
    pub messages: Vec<ChatMessage>,

    /// Alternate list of models for routing overrides.
    pub models: Option<Vec<String>>,

    /// Preferences for provider routing.
    pub provider: Option<ProviderPreferences>,

    /// Configuration for model reasoning/thinking tokens
    pub reasoning: Option<ReasoningConfig>,

    /// Whether to include usage information in the response
    pub usage: Option<UsageConfig>,

    /// List of prompt transforms (OpenRouter-only).
    pub transforms: Option<Vec<String>>,

    /// Enable streaming of results. Defaults to false
    pub stream: Option<bool>,

    /// Maximum number of tokens (range: [1, context_length)).
    pub max_tokens: Option<i32>,

    /// Sampling temperature (range: [0, 2]).
    pub temperature: Option<f64>,

    /// Seed for deterministic outputs.
    pub seed: Option<i32>,

    /// Top-p sampling value (range: (0, 1]).
    pub top_p: Option<f64>,

    /// Top-k sampling value (range: [1, Infinity)).
    pub top_k: Option<i32>,

    /// Frequency penalty (range: [-2, 2]).
    pub frequency_penalty: Option<f64>,

    /// Presence penalty (range: [-2, 2]).
    pub presence_penalty: Option<f64>,

    /// Repetition penalty (range: (0, 2]).
    pub repetition_penalty: Option<f64>,

    /// Mapping of token IDs to bias values.
    pub logit_bias: Option<HashMap<String, f64>>,

    /// Number of top log probabilities to return.
    pub top_logprobs: Option<i32>,

    /// Minimum probability threshold (range: [0, 1]).
    pub min_p: Option<f64>,

    /// Alternate top sampling parameter (range: [0, 1]).
    pub top_a: Option<f64>,

    /// A stable identifier for your end-users. Used to help detect and prevent abuse.
    pub user: Option<String>,
}

impl ChatRequest {
    /// Create a simple chat request with a single user message
    pub fn simple(model: impl Into<String>, message: impl Into<String>) -> Self {
        let user_message = ChatMessage {
            role: MessageRole::User,
            content: Content::Simple(message.into()),
        };
        Self::builder()
            .model(model.into())
            .messages(vec![user_message])
            .stream(false)
            .build()
    }

    /// Create a chat request with system prompt and user message
    pub fn with_system(
        model: impl Into<String>,
        system_prompt: impl Into<String>,
        user_message: impl Into<String>,
    ) -> Self {
        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: Content::Simple(system_prompt.into()),
            },
            ChatMessage {
                role: MessageRole::User,
                content: Content::Simple(user_message.into()),
            },
        ];
        Self::builder()
            .model(model.into())
            .messages(messages)
            .build()
    }

    /// Create a simple streaming chat request with a single user message
    pub fn simple_stream(model: impl Into<String>, message: impl Into<String>) -> Self {
        let user_message = ChatMessage {
            role: MessageRole::User,
            content: Content::Simple(message.into()),
        };
        Self::builder()
            .model(model.into())
            .messages(vec![user_message])
            .stream(true)
            .build()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// The role of the message author
    pub role: MessageRole,

    /// The message content
    pub content: Content,
}

impl ChatMessage {
    pub fn system(content: Content) -> Self {
        Self {
            role: MessageRole::System,
            content,
        }
    }

    pub fn user(content: Content) -> Self {
        Self {
            role: MessageRole::User,
            content,
        }
    }

    pub fn assistant(content: Content) -> Self {
        Self {
            role: MessageRole::Assistant,
            content,
        }
    }

    pub fn tool(content: Content) -> Self {
        Self {
            role: MessageRole::Tool,
            content,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    Simple(String),
    Complex(Vec<Part>),
}

impl Content {
    pub fn simple(content: impl Into<String>) -> Self {
        Content::Simple(content.into())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageUrl {
    url: String,
    detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct File {
    filename: String,
    file_data: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Part {
    Text { text: String },
    Image { image_url: ImageUrl },
    File { file: File },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    Developer,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPreferences {
    /// Sort preference (e.g., price, throughput).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningConfig {
    /// OpenAI-style reasoning effort setting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<ReasoningEffort>,

    /// Non-OpenAI-style reasoning effort setting. Cannot be used simultaneously with effort.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,

    /// Whether to exclude reasoning from the response. Defaults to false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageConfig {
    /// Whether to include usage information in the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<bool>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_constructor_methods() {
        // Test simple constructor
        let simple_request = ChatRequest::simple("gpt-3.5-turbo", "Hello world");
        assert_eq!(simple_request.model, "gpt-3.5-turbo");
        assert_eq!(simple_request.messages.len(), 1);
        assert!(matches!(simple_request.messages[0].role, MessageRole::User));
        assert_eq!(simple_request.stream, Some(false));

        // Test with_system constructor
        let system_request = ChatRequest::with_system("gpt-4", "You are helpful", "How are you?");
        assert_eq!(system_request.model, "gpt-4");
        assert_eq!(system_request.messages.len(), 2);
        assert!(matches!(
            system_request.messages[0].role,
            MessageRole::System
        ));
        assert!(matches!(system_request.messages[1].role, MessageRole::User));

        // Test new constructor with custom messages
        let custom_messages = vec![ChatMessage {
            role: MessageRole::Developer,
            content: Content::Simple("Debug mode on".to_string()),
        }];
        let custom_request = ChatRequest::builder()
            .model("claude-3".into())
            .messages(custom_messages)
            .build();
        assert_eq!(custom_request.model, "claude-3");
        assert_eq!(custom_request.messages.len(), 1);
        assert!(matches!(
            custom_request.messages[0].role,
            MessageRole::Developer
        ));
    }

    #[test]
    fn test_simple_chat_request_serialization() {
        let request = ChatRequest::simple("gpt-3.5-turbo", "Hello world");

        // Test that we can serialize the request (this would normally be sent to the API)
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("gpt-3.5-turbo"));
        assert!(json.contains("Hello world"));
    }

    #[tokio::test]
    async fn test_chat_request_serialization() {
        let request = ChatRequest::builder()
            .model("gpt-4".into())
            .messages(vec![
                ChatMessage {
                    role: MessageRole::System,
                    content: Content::Simple("You are a helpful assistant.".into()),
                },
                ChatMessage {
                    role: MessageRole::User,
                    content: Content::Simple("Hello! How can you help me today?".into()),
                },
            ])
            .provider(ProviderPreferences {
                sort: Some("price".to_string()),
            })
            .reasoning(ReasoningConfig {
                effort: Some(ReasoningEffort::High),
                max_tokens: None,
                exclude: Some(false),
            })
            .usage(UsageConfig {
                include: Some(true),
            })
            .max_tokens(150)
            .temperature(0.7)
            .seed(42)
            .top_p(0.9)
            .frequency_penalty(0.1)
            .presence_penalty(0.1)
            .user("test_user_123".to_string())
            .build();

        let json = serde_json::to_string_pretty(&request).unwrap();
        println!("Serialized chat request:\n{}", json);

        // Test deserialization
        let deserialized: ChatRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.model, "gpt-4");
        assert_eq!(deserialized.messages.len(), 2);
        assert_eq!(deserialized.max_tokens, Some(150));
        assert_eq!(deserialized.temperature, Some(0.7));
    }

    #[tokio::test]
    async fn test_chat_message_simple_content() {
        let message = ChatMessage {
            role: MessageRole::User,
            content: Content::Simple("Hello world!".to_string()),
        };

        let json = serde_json::to_string(&message).unwrap();
        let deserialized: ChatMessage = serde_json::from_str(&json).unwrap();

        match deserialized.content {
            Content::Simple(text) => assert_eq!(text, "Hello world!"),
            _ => panic!("Expected simple content"),
        }
        assert!(matches!(deserialized.role, MessageRole::User));
    }

    #[tokio::test]
    async fn test_message_roles_serialization() {
        let roles = vec![
            MessageRole::System,
            MessageRole::Developer,
            MessageRole::User,
            MessageRole::Assistant,
            MessageRole::Tool,
        ];

        for role in roles {
            let message = ChatMessage {
                role: role.clone(),
                content: Content::Simple("test".to_string()),
            };

            let json = serde_json::to_string(&message).unwrap();
            let deserialized: ChatMessage = serde_json::from_str(&json).unwrap();

            // Test that roles serialize/deserialize correctly
            match (&role, &deserialized.role) {
                (MessageRole::System, MessageRole::System) => (),
                (MessageRole::Developer, MessageRole::Developer) => (),
                (MessageRole::User, MessageRole::User) => (),
                (MessageRole::Assistant, MessageRole::Assistant) => (),
                (MessageRole::Tool, MessageRole::Tool) => (),
                _ => panic!("Role mismatch: {:?} != {:?}", role, deserialized.role),
            }
        }
    }

    #[tokio::test]
    async fn test_reasoning_effort_serialization() {
        let efforts = vec![
            ReasoningEffort::High,
            ReasoningEffort::Medium,
            ReasoningEffort::Low,
        ];

        for effort in efforts {
            let config = ReasoningConfig {
                effort: Some(effort.clone()),
                max_tokens: None,
                exclude: None,
            };

            let json = serde_json::to_string(&config).unwrap();
            let deserialized: ReasoningConfig = serde_json::from_str(&json).unwrap();

            assert!(deserialized.effort.is_some());
            match (&effort, deserialized.effort.as_ref().unwrap()) {
                (ReasoningEffort::High, ReasoningEffort::High) => (),
                (ReasoningEffort::Medium, ReasoningEffort::Medium) => (),
                (ReasoningEffort::Low, ReasoningEffort::Low) => (),
                _ => panic!("Effort mismatch"),
            }
        }
    }

    #[tokio::test]
    async fn test_minimal_chat_request() {
        let request = ChatRequest::simple("gpt-3.5-turbo", "What is Rust?");

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ChatRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.model, "gpt-3.5-turbo");
        assert_eq!(deserialized.messages.len(), 1);
        assert!(deserialized.max_tokens.is_none());
        assert!(deserialized.temperature.is_none());
    }
}
