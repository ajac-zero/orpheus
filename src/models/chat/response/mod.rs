mod completion;
mod stream;
mod usage;

pub use super::*;
pub use completion::*;
pub use stream::*;
pub use usage::*;

#[cfg(test)]
mod test {
    use super::{
        super::{Content, Role},
        *,
    };

    #[tokio::test]
    async fn test_chat_response_deserialization() {
        let response_json = r#"{
                "id": "chatcmpl-abc123",
                "choices": [
                    {
                        "index": 0,
                        "message": {
                            "role": "assistant",
                            "content": "Hello! I'm doing well, thank you for asking. How can I assist you today?"
                        },
                        "finish_reason": "stop"
                    }
                ],
                "provider": "OpenAI",
                "model": "openai/gpt-4o",
                "usage": {
                    "prompt_tokens": 10,
                    "completion_tokens": 20,
                    "total_tokens": 30
                }
            }"#;

        let response: ChatCompletion = serde_json::from_str(response_json).unwrap();
        assert_eq!(response.id, "chatcmpl-abc123".to_string());

        let choices = response.choices;
        assert_eq!(choices.len(), 1);

        let message = choices[0].message.to_owned();
        assert_eq!(message.role, Role::Assistant);
        assert_eq!(
            message.content,
            Content::Simple(
                "Hello! I'm doing well, thank you for asking. How can I assist you today?"
                    .to_string()
            )
        );
    }

    #[tokio::test]
    async fn test_chat_stream_chunk_deserialization() {
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

    #[tokio::test]
    async fn test_chat_stream_chunk_with_usage() {
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
