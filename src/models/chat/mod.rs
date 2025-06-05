mod request;
mod response;

pub use request::*;
pub use response::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

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

        let request = ChatRequest::builder()
            .model("gpt-4".into())
            .messages(messages.clone())
            .max_tokens(200)
            .temperature(0.7)
            .build();

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
        let request = ChatRequest::builder()
            .model("gpt-4".into())
            .messages(vec![
                ChatMessage {
                    role: MessageRole::System,
                    content: Content::Simple("You are a creative writing assistant.".to_string()),
                },
                ChatMessage {
                    role: MessageRole::User,
                    content: Content::Simple("Write a short haiku about programming.".to_string()),
                },
            ])
            .usage(UsageConfig {
                include: Some(true),
            })
            .max_tokens(100)
            .temperature(0.8)
            .top_p(0.9)
            .user("creative_writer_001".into())
            .build();

        // Example of how you would make the HTTP request
        // Uncomment and modify when you have a real endpoint:
        let client = reqwest::Client::new();
        let response = client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header(
                "Authorization",
                "Bearer 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",
            )
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
