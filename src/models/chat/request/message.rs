use serde::{Deserialize, Serialize};

use super::content::{Content, Part};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    /// The role of the message author
    pub role: Role,

    /// The message content
    pub content: Content,

    pub tool_calls: Option<Vec<ToolCall>>,

    pub annotations: Option<Vec<Annotation>>,
}

impl Message {
    pub fn new(role: Role, content: Content) -> Self {
        Self {
            role,
            content,
            tool_calls: None,
            annotations: None,
        }
    }

    /// Create a new `Message` with role `Role::System`.
    ///
    /// # Example
    ///
    /// ```
    /// use orpheus::{Message, Role};
    ///
    /// let system_message = Message::system("You are an AI");
    /// let generic_message = Message::new(Role::System, "You are an AI".into());
    ///
    /// assert_eq!(system_message, generic_message);
    /// ```
    pub fn system(content: impl Into<Content>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
            tool_calls: None,
            annotations: None,
        }
    }

    pub fn user(content: impl Into<Content>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            tool_calls: None,
            annotations: None,
        }
    }

    pub fn assistant(content: impl Into<Content>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            tool_calls: None,
            annotations: None,
        }
    }

    pub fn tool(content: impl Into<Content>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            tool_calls: None,
            annotations: None,
        }
    }

    pub fn with_image(mut self, url: impl Into<String>, detail: Option<String>) -> Self {
        let image_part = Part::image_url(url.into(), detail);
        self.content = self.content.add_part(image_part);
        self
    }

    pub fn with_file(mut self, filename: impl Into<String>, data: impl Into<String>) -> Self {
        let file_part = Part::file(filename.into(), data.into());
        self.content = self.content.add_part(file_part);
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    Developer,
    User,
    Assistant,
    Tool,
}

impl Role {
    pub fn from_string(string: &str) -> Result<Self, String> {
        match string {
            "system" => Ok(Role::System),
            "developer" => Ok(Role::Developer),
            "user" => Ok(Role::User),
            "assistant" => Ok(Role::Assistant),
            "tool" => Ok(Role::Tool),
            _ => Err(format!("Invalid message role: {}", string)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolCall {
    Function { id: String, function: Function },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Function {
    name: String,
    arguments: String,
}

impl Function {
    pub fn is(&self, name: &str) -> bool {
        self.name == name
    }

    pub fn get_args<T: serde::de::DeserializeOwned>(&self) -> serde_json::Result<T> {
        serde_json::from_str(&self.arguments)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Annotation {
    UrlCitation { url_citation: UrlCitation },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UrlCitation {
    url: String,
    title: String,
    content: Option<String>,
    start_index: u64,
    end_index: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessages(Vec<Message>);

impl From<Vec<Message>> for ChatMessages {
    fn from(value: Vec<Message>) -> Self {
        ChatMessages(value)
    }
}

impl<const N: usize> From<[Message; N]> for ChatMessages {
    fn from(value: [Message; N]) -> Self {
        ChatMessages(value.to_vec())
    }
}

impl From<String> for ChatMessages {
    fn from(value: String) -> Self {
        ChatMessages(vec![Message::from(value)])
    }
}

impl From<Message> for ChatMessages {
    fn from(value: Message) -> Self {
        ChatMessages(vec![value])
    }
}

impl From<&str> for ChatMessages {
    fn from(value: &str) -> Self {
        ChatMessages(vec![Message::from(value)])
    }
}
