use serde::{Deserialize, Serialize};

use super::content::Content;

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// The role of the message author
    pub role: MessageRole,

    /// The message content
    pub content: Content,

    pub tool_calls: Option<Vec<ToolCall>>,

    pub annotations: Option<Vec<Annotations>>,
}

impl ChatMessage {
    pub fn new(role: MessageRole, content: Content) -> Self {
        Self {
            role,
            content,
            tool_calls: None,
            annotations: None,
        }
    }

    pub fn system(content: impl Into<Content>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
            tool_calls: None,
            annotations: None,
        }
    }

    pub fn user(content: impl Into<Content>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
            tool_calls: None,
            annotations: None,
        }
    }

    pub fn assistant(content: impl Into<Content>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            tool_calls: None,
            annotations: None,
        }
    }

    pub fn tool(content: impl Into<Content>) -> Self {
        Self {
            role: MessageRole::Tool,
            content: content.into(),
            tool_calls: None,
            annotations: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    Developer,
    User,
    Assistant,
    Tool,
}

impl MessageRole {
    pub fn from_string(string: &str) -> Result<Self, String> {
        match string {
            "system" => Ok(MessageRole::System),
            "developer" => Ok(MessageRole::Developer),
            "user" => Ok(MessageRole::User),
            "assistant" => Ok(MessageRole::Assistant),
            "tool" => Ok(MessageRole::Tool),
            _ => Err(format!("Invalid message role: {}", string)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolCall {
    Function { id: String, function: Function },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    name: String,
    arguments: String,
}

impl Function {
    pub fn is(&self, name: &str) -> bool {
        self.name == name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Annotations {
    UrlCitation { url_citation: UrlCitation },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlCitation {
    url: String,
    title: String,
    content: Option<String>,
    start_index: u64,
    end_index: u64,
}
