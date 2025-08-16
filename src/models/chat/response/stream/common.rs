use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatStreamChunk {
    /// Unique identifier for the chat completion
    pub id: String,

    /// The provider of the model
    pub provider: Option<String>,

    /// The model used for the completion
    pub model: Option<String>,

    /// The object type (always "chat.completion.chunk" for streaming)
    pub object: String,

    /// Unix timestamp of when the completion was created
    pub created: i64,

    /// List of streaming choices
    pub choices: Vec<ChatStreamChoice>,

    /// System fingerprint for the response
    pub system_fingerprint: Option<String>,

    /// Usage statistics (only present in the final chunk)
    pub usage: Option<super::ChatUsage>,
}

impl ChatStreamChunk {
    pub fn delta(&self) -> Result<&super::Message> {
        let message = &self
            .choices
            .iter()
            .next()
            .ok_or(Error::malformed_response(
                "Choices array in response is empty",
            ))?
            .delta;

        Ok(message)
    }

    pub fn into_delta(self) -> Result<super::Message> {
        let message = self
            .choices
            .into_iter()
            .next()
            .ok_or(Error::malformed_response(
                "Choices array in response is empty",
            ))?
            .delta;

        Ok(message)
    }

    pub fn into_content(self) -> Result<super::Content> {
        Ok(self.into_delta()?.content)
    }

    pub fn content(&self) -> Result<&super::Content> {
        Ok(&self.delta()?.content)
    }

    pub fn reasoning(&self) -> Result<Option<&String>> {
        Ok(self.delta()?.reasoning.as_ref())
    }
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatStreamChoice {
    /// The index of the choice
    pub index: u8,

    /// The delta containing incremental message content
    pub delta: super::Message,

    /// The reason the completion finished
    pub finish_reason: Option<String>,

    /// The native finish reason from the provider
    pub native_finish_reason: Option<String>,

    /// Log probabilities for the choice
    pub logprobs: Option<serde_json::Value>,
}
