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
