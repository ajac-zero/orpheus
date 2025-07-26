use bon::Builder;
use serde::{Deserialize, Serialize};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[builder(state_mod(vis = "pub(crate)"))]
pub struct ReasoningConfig {
    /// OpenAI-style reasoning effort setting
    pub effort: Option<Effort>,

    /// Non-OpenAI-style reasoning effort setting. Cannot be used simultaneously with effort.
    pub max_tokens: Option<i32>,

    /// Whether to exclude reasoning from the response. Defaults to false
    pub exclude: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Effort {
    High,
    Medium,
    Low,
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn test_no_reasoning() {
        let client = Orpheus::from_env().unwrap();

        let response = client
            .chat("what is 2 + 2 in latin")
            .model("google/gemini-2.5-flash-lite")
            .send()
            .unwrap()
            .into_message()
            .unwrap();

        assert!(response.reasoning.is_none());
    }

    #[test]
    fn test_set_reasoning_effort() {
        let client = Orpheus::from_env().unwrap();

        let response = client
            .chat("what is 2 + 2 in latin")
            .model("google/gemini-2.5-flash-lite")
            .with_reasoning(|r| r.effort(Effort::Low))
            .send()
            .unwrap()
            .into_message()
            .unwrap();

        assert!(response.reasoning.is_some());
    }

    #[test]
    fn test_set_reasoning_budget() {
        let client = Orpheus::from_env().unwrap();

        let response = client
            .chat("what is 2 + 2 in latin")
            .model("google/gemini-2.5-flash-lite")
            .with_reasoning(|r| r.max_tokens(100))
            .send()
            .unwrap()
            .into_message()
            .unwrap();

        assert!(response.reasoning.is_some());
    }
}
