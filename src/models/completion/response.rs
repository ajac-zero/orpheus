use serde::{Deserialize, Serialize};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Unique identifier for the completion
    pub id: Option<String>,

    /// List of completion choices
    pub choices: Option<Vec<CompletionChoice>>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChoice {
    /// The generated text completion
    pub text: Option<String>,

    /// The index of this choice in the list
    pub index: Option<i32>,

    /// The reason why the completion finished
    pub finish_reason: Option<String>,
}

#[cfg(test)]
mod test {
    use super::*;

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
}
