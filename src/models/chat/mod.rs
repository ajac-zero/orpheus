mod request;
mod response;

pub use request::*;
pub use response::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::*;
    use futures_lite::StreamExt;
    use reqwest::header::CONTENT_TYPE;
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
        let api_key = std::env::var(API_KEY_ENV_VAR).expect("load env var");
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{BASE_URL_ENV_VAR}{CHAT_COMPLETION_PATH}"))
            .header(CONTENT_TYPE, "application/json")
            .bearer_auth(api_key)
            .json(&request)
            .send()
            .await
            .unwrap();

        let chat_response: ChatCompletion = response.json().await.unwrap();
        println!("Chat Response: {:?}", chat_response);

        // For now, just test that we can serialize the request
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("gpt-4"));
        assert!(json.contains("Write a short haiku about programming"));
        assert!(json.contains("creative_writer_001"));
    }

    #[tokio::test]
    async fn test_http_streaming_request() {
        // Create a streaming chat request
        let request = ChatRequest::builder()
            .model("gpt-4".into())
            .messages(vec![
                ChatMessage::system(Content::simple("You are a helpful assistant.")),
                ChatMessage::user(Content::simple("Say hello!")),
            ])
            .stream(true) // Enable streaming
            .temperature(0.7)
            .build();

        // Verify the request is properly configured for streaming
        assert_eq!(request.stream, Some(true));

        // Test serialization
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"stream\":true"));
        assert!(json.contains("Say hello!"));

        // Example of how you would make the streaming HTTP request
        // Note: In a real test, you'd need a mock server or test against actual API
        let api_key = std::env::var(API_KEY_ENV_VAR).unwrap();
        let client = reqwest::Client::new();

        // This is how you would set up the streaming request
        let response = client
            .post(format!("{BASE_URL_ENV_VAR}{CHAT_COMPLETION_PATH}"))
            .header(CONTENT_TYPE, "application/json")
            .bearer_auth(api_key)
            .json(&request)
            .send()
            .await
            .unwrap();

        let mut accumulated_content = String::new();
        let mut is_finished = false;

        let mut stream = response.bytes_stream();

        // Process mock SSE lines
        while let Some(Ok(bytes)) = stream.next().await {
            let data = String::from_utf8(bytes.to_vec()).unwrap();

            for line in data.lines() {
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
        }

        assert!(is_finished);
        println!(
            "Successfully processed streaming chat completion: '{}'",
            accumulated_content
        );
    }
}
