use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Unique identifier for the completion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// List of completion choices
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<CompletionChoice>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChoice {
    /// The generated text completion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// The index of this choice in the list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i32>,

    /// The reason why the completion finished
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}
