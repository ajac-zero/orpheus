mod request;
mod response;

pub use request::*;
pub use response::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[tokio::test]
    async fn test_chat_request_serialization() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![
                ChatMessage {
                    role: MessageRole::System,
                    content: Content::Simple("You are a helpful assistant.".to_string()),
                },
                ChatMessage {
                    role: MessageRole::User,
                    content: Content::Simple("Hello! How can you help me today?".to_string()),
                },
            ],
            models: None,
            provider: Some(ProviderPreferences {
                sort: Some("price".to_string()),
            }),
            reasoning: Some(ReasoningConfig {
                effort: Some(ReasoningEffort::High),
                max_tokens: None,
                exclude: Some(false),
            }),
            usage: Some(UsageConfig {
                include: Some(true),
            }),
            transforms: None,
            stream: Some(false),
            max_tokens: Some(150),
            temperature: Some(0.7),
            seed: Some(42),
            top_p: Some(0.9),
            top_k: None,
            frequency_penalty: Some(0.1),
            presence_penalty: Some(0.1),
            repetition_penalty: None,
            logit_bias: None,
            top_logprobs: None,
            min_p: None,
            top_a: None,
            user: Some("test_user_123".to_string()),
        };

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
    async fn test_chat_response_deserialization() {
        let response_json = r#"{
            "id": "chatcmpl-abc123",
            "choices": [
                {
                    "message": {
                        "role": "assistant",
                        "content": "Hello! I'm doing well, thank you for asking. How can I assist you today?"
                    }
                }
            ]
        }"#;

        let response: ChatResponse = serde_json::from_str(response_json).unwrap();
        assert_eq!(response.id, Some("chatcmpl-abc123".to_string()));
        assert!(response.choices.is_some());

        let choices = response.choices.unwrap();
        assert_eq!(choices.len(), 1);

        let message = choices[0].message.as_ref().unwrap();
        assert_eq!(message.role, MessageRole::Assistant);
        assert_eq!(
            message.content,
            Content::Simple("Hello! I'm doing well, thank you for asking. How can I assist you today?".to_string())
        );
    }

    #[tokio::test]
    async fn test_minimal_chat_request() {
        let request = ChatRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Content::Simple("What is Rust?".to_string()),
            }],
            models: None,
            provider: None,
            reasoning: None,
            usage: None,
            transforms: None,
            stream: None,
            max_tokens: None,
            temperature: None,
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

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ChatRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.model, "gpt-3.5-turbo");
        assert_eq!(deserialized.messages.len(), 1);
        assert!(deserialized.max_tokens.is_none());
        assert!(deserialized.temperature.is_none());
    }

    #[tokio::test]
    async fn test_conversation_flow() {
        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: Content::Simple("You are a helpful coding assistant.".to_string()),
            },
            ChatMessage {
                role: MessageRole::User,
                content: Content::Simple("Explain what Rust is.".to_string()),
            },
            ChatMessage {
                role: MessageRole::Assistant,
                content: Content::Simple("Rust is a systems programming language...".to_string()),
            },
            ChatMessage {
                role: MessageRole::User,
                content: Content::Simple("Can you show me a simple example?".to_string()),
            },
        ];

        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: messages.clone(),
            models: None,
            provider: None,
            reasoning: None,
            usage: None,
            transforms: None,
            stream: Some(false),
            max_tokens: Some(200),
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

        let json = serde_json::to_string_pretty(&request).unwrap();
        println!("Conversation request:\n{}", json);

        let deserialized: ChatRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.messages.len(), 4);

        // Verify conversation order is preserved
        match &deserialized.messages[0].role {
            MessageRole::System => (),
            _ => panic!("First message should be system"),
        }
        match &deserialized.messages[1].role {
            MessageRole::User => (),
            _ => panic!("Second message should be user"),
        }
        match &deserialized.messages[2].role {
            MessageRole::Assistant => (),
            _ => panic!("Third message should be assistant"),
        }
        match &deserialized.messages[3].role {
            MessageRole::User => (),
            _ => panic!("Fourth message should be user"),
        }
    }

    #[tokio::test]
    async fn test_http_request_example() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![
                ChatMessage {
                    role: MessageRole::System,
                    content: Content::Simple("You are a creative writing assistant.".to_string()),
                },
                ChatMessage {
                    role: MessageRole::User,
                    content: Content::Simple("Write a short haiku about programming.".to_string()),
                },
            ],
            models: None,
            provider: None,
            reasoning: None,
            usage: Some(UsageConfig {
                include: Some(true),
            }),
            transforms: None,
            stream: Some(false),
            max_tokens: Some(100),
            temperature: Some(0.8),
            seed: None,
            top_p: Some(0.9),
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
            repetition_penalty: None,
            logit_bias: None,
            top_logprobs: None,
            min_p: None,
            top_a: None,
            user: Some("creative_writer_001".to_string()),
        };

        // Example of how you would make the HTTP request
        // Uncomment and modify when you have a real endpoint:
        let client = reqwest::Client::new();
        let response = client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", "Bearer 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")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .unwrap();

        let chat_response: ChatResponse = response.json().await.unwrap();
        println!("Chat Response: {:?}", chat_response);

        // For now, just test that we can serialize the request
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("gpt-4"));
        assert!(json.contains("Write a short haiku about programming"));
        assert!(json.contains("creative_writer_001"));
    }
}
