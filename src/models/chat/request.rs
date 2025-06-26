use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// The model ID to use. If unspecified, the user's default is used.
    pub model: String,

    /// List of messages in the conversation
    pub messages: Vec<ChatMessage>,

    /// Alternate list of models for routing overrides.
    pub models: Option<Vec<String>>,

    pub plugins: Option<Vec<Plugins>>,

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
    pub fn new(
        model: String,
        messages: Vec<ChatMessage>,
        models: Option<Vec<String>>,
        plugins: Option<Vec<Plugins>>,
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
    ) -> Self {
        Self {
            model,
            messages,
            models,
            plugins,
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
        }
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

    pub fn with_image(self, url: impl Into<String>, detail: Option<String>) -> Self {
        let image_part = Part::image_url(url.into(), detail);
        self.add_part(image_part)
    }

    pub fn with_file(self, filename: impl Into<String>, data: impl Into<String>) -> Self {
        let file_part = Part::file(filename.into(), data.into());
        self.add_part(file_part)
    }

    /// Consumes the current content and creates a new content with the appended part.
    /// 1. `Self::Simple` variant is transformed into a complex variant with the original text prepended as a "text" part.
    /// 2. `Self::Complex` variant is modified by appending the new part to the existing parts vector.
    fn add_part(self, part: Part) -> Self {
        let new_parts = match self {
            Self::Simple(string) => vec![Part::text(string), part],
            Self::Complex(mut parts) => {
                parts.push(part);
                parts
            }
        };
        Content::Complex(new_parts)
    }

    pub fn to_string(&self) -> String {
        match self {
            Content::Simple(s) => s.clone(),
            Content::Complex(_) => todo!(),
        }
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
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Part {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
    File { file: File },
}

impl Part {
    pub fn text(string: String) -> Self {
        Self::Text { text: string }
    }

    pub fn image_url(url: String, detail: Option<String>) -> Self {
        Self::ImageUrl {
            image_url: ImageUrl { url, detail },
        }
    }

    pub fn file(filename: String, data: String) -> Self {
        Self::File {
            file: File {
                filename,
                file_data: data,
            },
        }
    }
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

impl MessageRole {
    pub fn from_string(string: &str) -> Result<Self, String> {
        match string {
            "system" => Ok(MessageRole::System),
            "developer" => Ok(MessageRole::Developer),
            "user" => Ok(MessageRole::User),
            "assistant" => Ok(MessageRole::Assistant),
            "tool" => Ok(MessageRole::Tool),
            _ => Err(format!("Invalid message role: {}", string)),
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "engine", rename_all = "kebab-case")]
pub enum ParsingEngine {
    PdfText,
    MistralOcr,
    Native,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "id", rename_all = "kebab-case")]
pub enum Plugins {
    FileParser {
        pdf: ParsingEngine,
    },
    Web {
        max_results: Option<u64>,
        search_prompt: Option<String>,
    },
}

#[cfg(test)]
mod test {
    use serde_json::{from_value, json};

    use super::*;

    #[test]
    fn test_chat_message_simple_content() {
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

    #[test]
    fn test_message_roles_serialization() {
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

    #[test]
    fn test_reasoning_effort_serialization() {
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

    #[test]
    fn test_simple_message_deserialization() {
        let data = json!(                {
                    "role": "user",
                    "content": "hello!"
        });

        let model = from_value::<ChatMessage>(data).unwrap();
        println!("Chat Message: {:?}", model);
    }

    #[test]
    fn test_text_type_message_deserialization() {
        let data = json!(                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": "hii!"
                        }
                    ]
        });

        let model = from_value::<ChatMessage>(data).unwrap();
        println!("Chat Message: {:?}", model);
    }

    #[test]
    fn test_image_type_message_deserialization() {
        let data = json!(                {
                    "role": "user",
                    "content": [
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": "https://upload.wikimedia.org/wikipedia/commons/thumb/d/dd/Gfp-wisconsin-madison-the-nature-boardwalk.jpg/2560px-Gfp-wisconsin-madison-the-nature-boardwalk.jpg"
                            }
                        }
                    ]
        });

        let model = from_value::<ChatMessage>(data).unwrap();
        println!("Chat Message: {:?}", model);
    }

    #[test]
    fn test_complex_request_deserialization() {
        let data = json!({
            "model": "google/gemini-2.0-flash-001",
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": "What's in this image?"
                        },
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": "https://upload.wikimedia.org/wikipedia/commons/thumb/d/dd/Gfp-wisconsin-madison-the-nature-boardwalk.jpg/2560px-Gfp-wisconsin-madison-the-nature-boardwalk.jpg"
                            }
                        }
                    ]
                }
            ]
        });

        let model = from_value::<ChatRequest>(data).unwrap();
        println!("Complex Chat Message: {:?}", model);
    }

    #[test]
    fn test_file_parser_plugin_deserialize() {
        let payload = json!({
              "id": "file-parser",
              "pdf": {
                "engine": "pdf-text", // or 'mistral-ocr' or 'native'
              },
        });

        let plugin = from_value::<Plugins>(payload).unwrap();
        println!("Web Plugin: {:?}", plugin);

        // assert that the plugin is of variant FileParser
        match plugin {
            Plugins::FileParser { pdf } => {
                assert!(matches!(pdf, ParsingEngine::PdfText));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_web_plugin_with_params_deserialize() {
        let payload = json!({"id": "web" });

        let plugin = from_value::<Plugins>(payload).unwrap();
        println!("Web Plugin: {:?}", plugin);

        // assert that the plugin is of variant FileParser
        match plugin {
            Plugins::Web {
                max_results,
                search_prompt,
            } => {
                assert!(max_results.is_none());
                assert!(search_prompt.is_none());
            }
            _ => unreachable!(),
        }

        let payload =
            json!({"id": "web", "max_results": 10, "search_prompt": "Some relevant web results:" });

        let plugin = from_value::<Plugins>(payload).unwrap();
        println!("Web Plugin: {:?}", plugin);

        // assert that the plugin is of variant FileParser
        match plugin {
            Plugins::Web {
                max_results,
                search_prompt,
            } => {
                assert!(max_results == Some(10));
                assert!(search_prompt == Some("Some relevant web results:".to_string()));
            }
            _ => unreachable!(),
        }
    }
}
