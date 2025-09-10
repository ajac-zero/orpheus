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

#[cfg(test)]
mod test {
    use super::*;
    use crate::models::chat::Role;

    #[test]
    fn completion_chunk_deserialization() {
        let chunk_json = r#"{
            "id": "gen-1749454386-VRY5G3UxpEJ8uAfP2MnL",
            "provider": "OpenAI",
            "model": "openai/gpt-4o",
            "object": "chat.completion.chunk",
            "created": 1749454386,
            "choices": [
                {
                    "index": 0,
                    "delta": {
                        "role": "assistant",
                        "content": "Hello"
                    },
                    "finish_reason": null,
                    "native_finish_reason": null,
                    "logprobs": null
                }
            ],
            "system_fingerprint": "fp_07871e2ad8"
        }"#;

        let chunk: ChatStreamChunk = serde_json::from_str(chunk_json).unwrap();
        assert_eq!(chunk.id, "gen-1749454386-VRY5G3UxpEJ8uAfP2MnL");
        assert_eq!(chunk.provider, Some("OpenAI".to_string()));
        assert_eq!(chunk.model, Some("openai/gpt-4o".to_string()));
        assert_eq!(chunk.object, "chat.completion.chunk");
        assert_eq!(chunk.created, 1749454386);
        assert_eq!(chunk.choices.len(), 1);

        let choice = &chunk.choices[0];
        assert_eq!(choice.index, 0);
        assert_eq!(choice.delta.role, Role::Assistant);
        assert_eq!(choice.delta.content, "Hello".into());
        assert_eq!(choice.finish_reason, None);
    }

    #[test]
    fn completion_chunk_with_usage_deserialization() {
        let chunk_json = r#"{
            "id": "gen-1749454386-VRY5G3UxpEJ8uAfP2MnL",
            "provider": "OpenAI",
            "model": "openai/gpt-4o",
            "object": "chat.completion.chunk",
            "created": 1749454386,
            "choices": [
                {
                    "index": 0,
                    "delta": {
                        "role": "assistant",
                        "content": ""
                    },
                    "finish_reason": null,
                    "native_finish_reason": null,
                    "logprobs": null
                }
            ],
            "usage": {
                "prompt_tokens": 8,
                "completion_tokens": 9,
                "total_tokens": 17,
                "prompt_tokens_details": {
                    "cached_tokens": 0
                },
                "completion_tokens_details": {
                    "reasoning_tokens": 0
                }
            }
        }"#;

        let chunk: ChatStreamChunk = serde_json::from_str(chunk_json).unwrap();
        assert!(chunk.usage.is_some());

        let usage = chunk.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 8);
        assert_eq!(usage.completion_tokens, 9);
        assert_eq!(usage.total_tokens, 17);
        assert!(usage.prompt_tokens_details.is_some());
        assert!(usage.completion_tokens_details.is_some());
    }
}
