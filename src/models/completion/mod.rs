mod request;
mod response;

pub use request::*;
pub use response::*;

use crate::constants::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[tokio::test]
    async fn test_http_request_example() {
        // This is a mock test - replace with real endpoint when available
        let request = CompletionRequest::builder()
            .model("gpt-3.5-turbo".into())
            .prompt("Write a haiku about coding".into())
            .build();

        // Example of how you would make the HTTP request
        // Uncomment and modify when you have a real endpoint:
        let api_key = std::env::var(API_KEY_ENV_VAR).expect("load env var");
        let client = reqwest::Client::new();
        let response = client
            .post("https://openrouter.ai/api/v1/completions")
            .header("Content-Type", "application/json")
            .bearer_auth(api_key)
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
