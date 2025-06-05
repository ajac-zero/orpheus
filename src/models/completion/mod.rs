mod request;
mod response;

pub use request::*;
pub use response::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[tokio::test]
    async fn test_completion_request_serialization() {
        let mut request = CompletionRequest::new("gpt-3.5-turbo", "Hello, world!");
        request.max_tokens = Some(100);

        let json = serde_json::to_string_pretty(&request).unwrap();
        println!("Serialized request:\n{}", json);

        // Test deserialization
        let deserialized: CompletionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.model, "gpt-3.5-turbo");
        assert_eq!(deserialized.prompt, "Hello, world!");
        assert_eq!(deserialized.max_tokens, Some(100));
    }

    #[tokio::test]
    async fn test_completion_response_deserialization() {
        let response_json = r#"{
            "id": "cmpl-123",
            "choices": [
                {
                    "text": "Hello! How can I help you today?",
                    "index": 0,
                    "finish_reason": "stop"
                }
            ]
        }"#;

        let response: CompletionResponse = serde_json::from_str(response_json).unwrap();
        assert_eq!(response.id, Some("cmpl-123".to_string()));
        assert!(response.choices.is_some());

        let choices = response.choices.unwrap();
        assert_eq!(choices.len(), 1);
        assert_eq!(
            choices[0].text,
            Some("Hello! How can I help you today?".to_string())
        );
        assert_eq!(choices[0].index, Some(0));
        assert_eq!(choices[0].finish_reason, Some("stop".to_string()));
    }

    #[tokio::test]
    async fn test_http_request_example() {
        // This is a mock test - replace with real endpoint when available
        let request = CompletionRequest::new("gpt-3.5-turbo", "Write a haiku about coding");

        // Example of how you would make the HTTP request
        // Uncomment and modify when you have a real endpoint:
        let client = reqwest::Client::new();
        let response = client
            .post("https://openrouter.ai/api/v1/completions")
            .header(
                "Authorization",
                "Bearer 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",
            )
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .unwrap();

        let completion_response: CompletionResponse = response.json().await.unwrap();
        println!("Response: {:?}", completion_response);

        // For now, just test that we can serialize the request
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("gpt-3.5-turbo"));
        assert!(json.contains("Write a haiku about coding"));
    }
}
