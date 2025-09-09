use serde::{Deserialize, Serialize};

use crate::Error;

use super::content::{Content, Part};

/// Represents a message in a chat conversation with support for multimodal content.
///
/// Messages can contain text, images, and files, and are associated with different roles
/// (system, user, assistant, tool). The message structure supports advanced features like
/// tool calls, reasoning, and annotations.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    /// The role of the message author (system, user, assistant, tool, developer)
    pub role: Role,

    /// The message content, which can be simple text or complex multimodal content
    pub content: Content,

    /// Optional reasoning provided by the model (used by some reasoning models)
    pub reasoning: Option<String>,

    /// Tool call identifier when this message is a response to a tool call
    pub tool_call_id: Option<String>,

    /// Tool calls made by the assistant in this message
    pub tool_calls: Option<Vec<ToolCall>>,

    /// Additional annotations like citations or file references
    pub annotations: Option<Vec<Annotation>>,

    /// Detailed reasoning information for supported models
    pub reasoning_details: Option<Vec<Details>>,
}

impl Message {
    /// Creates a new message with the specified role and content.
    pub(crate) fn new(role: Role, content: Content) -> Self {
        Self {
            role,
            content,
            reasoning: None,
            tool_call_id: None,
            tool_calls: None,
            annotations: None,
            reasoning_details: None,
        }
    }

    /// Create a new `Message` with role `Role::System`.
    pub fn system(content: impl Into<Content>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
            reasoning: None,
            tool_call_id: None,
            tool_calls: None,
            annotations: None,
            reasoning_details: None,
        }
    }

    /// Creates a new message with the `User` role.
    pub fn user(content: impl Into<Content>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            reasoning: None,
            tool_call_id: None,
            tool_calls: None,
            annotations: None,
            reasoning_details: None,
        }
    }

    /// Creates a new message with the `Assistant` role.
    pub fn assistant(content: impl Into<Content>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            reasoning: None,
            tool_call_id: None,
            tool_calls: None,
            annotations: None,
            reasoning_details: None,
        }
    }

    /// Creates a new message with the `Tool` role.
    pub fn tool(id: impl Into<String>, content: impl Into<Content>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            reasoning: None,
            tool_call_id: Some(id.into()),
            tool_calls: None,
            annotations: None,
            reasoning_details: None,
        }
    }

    /// Adds an image to the message content.
    pub fn with_image(mut self, url: impl Into<String>) -> Self {
        let image_part = Part::image_url(url.into(), None);
        self.content = self.content.add_part(image_part);
        self
    }

    /// Adds a file to the message content.
    pub fn with_file(mut self, filename: impl Into<String>, data: impl Into<String>) -> Self {
        let file_part = Part::file(filename.into(), data.into());
        self.content = self.content.add_part(file_part);
        self
    }

    /// Adds audio input to the message content.
    pub fn with_audio(mut self, data: impl Into<String>, format: impl Into<String>) -> Self {
        let audio_part = Part::input_audio(data.into(), format.into());
        self.content = self.content.add_part(audio_part);
        self
    }
}

impl From<String> for Message {
    fn from(value: String) -> Self {
        Message::user(value)
    }
}

impl From<&str> for Message {
    fn from(value: &str) -> Self {
        Message::user(value)
    }
}

/// The role of a message author in a chat conversation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System message that sets context and behavior for the AI model
    System,
    /// Developer message (similar to system, may have provider-specific handling)
    Developer,
    /// User message containing prompts, questions, or instructions
    User,
    /// Assistant message containing AI model responses
    Assistant,
    /// Tool message containing the results of function/tool calls
    Tool,
}

impl TryFrom<String> for Role {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "system" => Ok(Role::System),
            "developer" => Ok(Role::Developer),
            "user" => Ok(Role::User),
            "assistant" => Ok(Role::Assistant),
            "tool" => Ok(Role::Tool),
            _ => Err(Error::parse_error("Invalid role")),
        }
    }
}

/// Represents a tool call made by the AI model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolCall {
    /// A function call with an ID and function details
    Function {
        /// Unique identifier for this tool call
        id: String,
        /// The function being called with its arguments
        function: Function,
    },
}

/// Represents a function call within a tool call.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Function {
    /// The name of the function to call
    pub name: String,
    /// The function arguments as a JSON string
    pub arguments: String,
}

/// Annotations that can be attached to messages for additional context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Annotation {
    UrlCitation { url_citation: UrlCitation },
    File { file: FileAnnotation },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileAnnotation {
    pub hash: String,
    pub name: String,
    pub content: Vec<Part>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UrlCitation {
    url: String,
    title: String,
    content: Option<String>,
    start_index: u64,
    end_index: u64,
}

/// A collection of messages that can be sent to an AI model.
///
/// This wrapper around `Vec<Message>` provides convenient conversion methods
/// from various input types including single messages, message arrays, and strings.
/// It's used internally by the chat API to handle different message input formats.
#[derive(Debug, Serialize, Deserialize)]
pub struct History(pub Vec<Message>);

impl History {
    pub fn iter(&self) -> std::slice::Iter<'_, Message> {
        self.0.iter()
    }
}

impl From<Vec<Message>> for History {
    fn from(value: Vec<Message>) -> Self {
        History(value)
    }
}

impl From<&Vec<Message>> for History {
    fn from(value: &Vec<Message>) -> Self {
        History(value.to_owned())
    }
}

impl<const N: usize> From<[Message; N]> for History {
    fn from(value: [Message; N]) -> Self {
        History(value.to_vec())
    }
}

impl From<String> for History {
    fn from(value: String) -> Self {
        History(vec![Message::from(value)])
    }
}

impl From<Message> for History {
    fn from(value: Message) -> Self {
        History(vec![value])
    }
}

impl From<&str> for History {
    fn from(value: &str) -> Self {
        History(vec![Message::from(value)])
    }
}

/// Detailed reasoning information provided by reasoning-capable models.
///
/// Some AI models provide detailed reasoning traces that explain their
/// thought process. This struct captures that information including
/// the format, text content, and optional signature verification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Details {
    /// The format of the reasoning details
    format: String,
    /// The detailed reasoning text
    text: String,
    /// Optional signature for verification of the reasoning trace
    signature: Option<String>,
}
