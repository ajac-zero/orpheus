use serde::{Deserialize, Serialize};

use super::content::{Content, Part};

/// Represents a message in a chat conversation with support for multimodal content.
///
/// Messages can contain text, images, and files, and are associated with different roles
/// (system, user, assistant, tool). The message structure supports advanced features like
/// tool calls, reasoning, and annotations.
///
/// # Examples
///
/// Creating a simple text message:
/// ```
/// use orpheus::prelude::*;
///
/// let message = Message::new(Role::User, "Hello!".into());
/// ```
///
/// Creating a multimodal message with image and file:
/// ```
/// use orpheus::prelude::*;
///
/// let message = Message::user("Analyze this data")
///     .with_image("https://example.com/chart.png")
///     .with_file("data.csv", "name,value\nA,10\nB,20");
/// ```
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
    ///
    /// This is the base constructor for creating messages. For convenience,
    /// consider using the role-specific methods like [`Message::user`],
    /// [`Message::system`], or [`Message::assistant`].
    ///
    /// # Arguments
    ///
    /// * `role` - The role of the message author
    /// * `content` - The content of the message (text, multimodal, etc.)
    ///
    /// # Examples
    ///
    /// ```
    /// use orpheus::prelude::*;
    /// use orpheus::models::chat::Content;
    ///
    /// let message = Message::new(Role::User, Content::simple("Hello"));
    /// let text_message = Message::new(Role::Assistant, "Response".into());
    /// ```
    pub fn new(role: Role, content: Content) -> Self {
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
    ///
    /// # Example
    ///
    /// ```
    /// use orpheus::prelude::*;
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
            reasoning: None,
            tool_call_id: None,
            tool_calls: None,
            annotations: None,
            reasoning_details: None,
        }
    }

    /// Creates a new message with the `User` role.
    ///
    /// This is a convenience method for creating user messages, which are typically
    /// the prompts or queries sent to the AI model.
    ///
    /// # Arguments
    ///
    /// * `content` - The content of the user message (text, multimodal, etc.)
    ///
    /// # Examples
    ///
    /// ```
    /// use orpheus::prelude::*;
    /// use orpheus::models::chat::{Content, Part};
    ///
    /// let simple_message = Message::user("What is the weather like?");
    /// let complex_content = Content::simple("Analyze this").add_part(
    ///     Part::image_url("https://example.com/image.jpg".to_string(), None)
    /// );
    /// let multimodal_message = Message::user(complex_content);
    /// ```
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
    ///
    /// This is typically used for AI model responses or when constructing
    /// conversation history that includes previous assistant messages.
    ///
    /// # Arguments
    ///
    /// * `content` - The content of the assistant message
    ///
    /// # Examples
    ///
    /// ```
    /// use orpheus::prelude::*;
    ///
    /// let message = Message::user("Hello, world!");
    /// ```
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
    ///
    /// Tool messages are used to provide the results of tool calls back to the model.
    /// They must include a tool call ID that matches a previous tool call request.
    ///
    /// # Arguments
    ///
    /// * `id` - The tool call ID this message is responding to
    /// * `content` - The content containing the tool's response
    ///
    /// # Examples
    ///
    /// ```
    /// use orpheus::prelude::*;
    ///
    /// let tool_response = Message::tool("call_123", "Function executed successfully");
    /// ```
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
    ///
    /// This method converts the message content to multimodal format if it isn't already,
    /// and appends an image URL part. The image will be processed by vision-capable models.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the image to include (HTTP or Base64)
    ///
    /// # Examples
    ///
    /// ```
    /// use orpheus::prelude::*;
    ///
    /// let message = Message::user("What's in this image?")
    ///     .with_image("https://example.com/photo.jpg");
    ///
    /// // Multiple images
    /// let comparison = Message::user("Compare these images")
    ///     .with_image("https://example.com/before.jpg")
    ///     .with_image("https://example.com/after.jpg");
    /// ```
    pub fn with_image(mut self, url: impl Into<String>) -> Self {
        let image_part = Part::image_url(url.into(), None);
        self.content = self.content.add_part(image_part);
        self
    }

    /// Adds a file to the message content.
    ///
    /// This method converts the message content to multimodal format if it isn't already,
    /// and appends a file part. Files can contain various types of data including PDFs,
    /// text documents, CSV data, JSON, code files, and more.
    ///
    /// # Arguments
    ///
    /// * `filename` - The name of the file (used for context and identification)
    /// * `data` - The content of the file as a string
    ///
    /// # Examples
    ///
    /// ```
    /// use orpheus::prelude::*;
    ///
    /// // CSV data analysis
    /// let csv_data = "name,age,city\nAlice,25,NYC\nBob,30,LA";
    /// let message = Message::user("Analyze this CSV data")
    ///     .with_file("users.csv", csv_data);
    ///
    /// // PDF document processing
    /// let pdf_content = "PDF content extracted as text...";
    /// let doc_message = Message::user("Summarize this document")
    ///     .with_file("report.pdf", pdf_content);
    ///
    /// // Multiple files
    /// let analysis = Message::user("Compare these datasets")
    ///     .with_file("data1.json", r#"{"values": [1,2,3]}"#)
    ///     .with_file("data2.json", r#"{"values": [4,5,6]}"#);
    /// ```
    pub fn with_file(mut self, filename: impl Into<String>, data: impl Into<String>) -> Self {
        let file_part = Part::file(filename.into(), data.into());
        self.content = self.content.add_part(file_part);
        self
    }

    /// Adds audio input to the message content.
    ///
    /// This method converts the message content to multimodal format if it isn't already,
    /// and appends an audio input part. The audio data must be base64-encoded and include
    /// the appropriate format specification. Audio-capable models can process speech,
    /// music, and other audio content.
    ///
    /// # Arguments
    ///
    /// * `data` - Base64-encoded audio data
    /// * `format` - Audio format specification (e.g., "wav", "mp3", "m4a", "flac", "webm")
    ///
    /// # Examples
    ///
    /// ```
    /// use orpheus::prelude::*;
    /// use orpheus::models::chat::Part;
    ///
    /// // WAV audio input
    /// let message = Message::user("What do you hear in this audio?")
    ///     .with_audio("base64_encoded_audio_data".to_string(), "wav".to_string());
    ///
    /// // MP3 audio for speech analysis
    /// let speech_message = Message::user("Transcribe this speech")
    ///     .with_audio("base64_mp3_data_here".to_string(), "mp3".to_string());
    ///
    /// // Multiple audio files for comparison
    /// let comparison = Message::user("Compare these two audio samples")
    ///     .with_audio("first_audio_base64".to_string(), "wav".to_string())
    ///     .with_audio("second_audio_base64".to_string(), "wav".to_string());
    ///
    /// // Combined multimodal content
    /// let multimodal_message = Message::user("Analyze both the image and audio")
    ///     .with_image("https://example.com/spectrogram.png")
    ///     .with_audio("audio_sample_base64".to_string(), "flac".to_string());
    /// ```
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
///
/// Different roles serve different purposes in the conversation flow:
/// - `System`: Sets the behavior and context for the AI
/// - `Developer`: Similar to system but may have different handling
/// - `User`: Represents the human user's input
/// - `Assistant`: Represents the AI model's responses
/// - `Tool`: Contains results from tool/function calls
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

impl Role {
    /// Creates a Role from a string representation.
    ///
    /// # Arguments
    ///
    /// * `string` - The string representation of the role (case-sensitive)
    ///
    /// # Returns
    ///
    /// * `Ok(Role)` - If the string matches a valid role
    /// * `Err(String)` - If the string doesn't match any valid role
    ///
    /// # Examples
    ///
    /// ```
    /// use orpheus::prelude::*;
    ///
    /// assert_eq!(Role::from_string("user").unwrap(), Role::User);
    /// assert_eq!(Role::from_string("assistant").unwrap(), Role::Assistant);
    /// assert!(Role::from_string("invalid").is_err());
    /// ```
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

/// Represents a tool call made by the AI model.
///
/// Tool calls allow the AI to execute functions and retrieve information
/// during the conversation. Currently only function calls are supported.
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
///
/// Contains the function name and its arguments as a JSON string.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Function {
    /// The name of the function to call
    pub name: String,
    /// The function arguments as a JSON string
    pub arguments: String,
}

/// Annotations that can be attached to messages for additional context.
///
/// Annotations provide metadata and references for message content,
/// such as citations and file references.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Annotation {
    /// A URL citation reference
    UrlCitation { url_citation: UrlCitation },
    /// A file reference annotation
    File { file: FileAnnotation },
}

/// Represents a file annotation attached to a message.
///
/// File annotations provide metadata and content for files referenced
/// in the conversation, including a hash for identification and the
/// actual content as a vector of parts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileAnnotation {
    /// Hash identifier for the file
    pub hash: String,
    /// Display name of the file
    pub name: String,
    /// The content of the file as a vector of parts
    pub content: Vec<Part>,
}

/// Represents a URL citation that references external content.
///
/// URL citations provide context and attribution for information
/// that comes from web sources, including the URL, title, and
/// the specific text range being referenced.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UrlCitation {
    /// The URL being cited
    url: String,
    /// The title of the referenced content
    title: String,
    /// Optional excerpt of the referenced content
    content: Option<String>,
    /// Starting character index of the citation in the message
    start_index: u64,
    /// Ending character index of the citation in the message
    end_index: u64,
}

/// A collection of chat messages that can be sent to an AI model.
///
/// This wrapper around `Vec<Message>` provides convenient conversion methods
/// from various input types including single messages, message arrays, and strings.
/// It's used internally by the chat API to handle different message input formats.
///
/// # Examples
///
/// ```
/// use orpheus::prelude::*;
/// use orpheus::models::chat::ChatMessages;
///
/// // From string
/// let messages: ChatMessages = "Hello, world!".into();
///
/// // From a vector of messages
/// let msg_vec = vec![
///     Message::system("You are a helpful assistant"),
///     Message::user("What is 2+2?")
/// ];
/// let messages: ChatMessages = msg_vec.into();
///
/// // From a single message
/// let single_message = Message::user("Hello");
/// let messages: ChatMessages = single_message.into();
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessages(Vec<Message>);

impl ChatMessages {
    /// Returns an iterator over the messages.
    ///
    /// # Examples
    ///
    /// ```
    /// use orpheus::prelude::*;
    /// use orpheus::models::chat::ChatMessages;
    ///
    /// let messages = ChatMessages::from(vec![
    ///     Message::user("First message"),
    ///     Message::user("Second message"),
    /// ]);
    ///
    /// for message in messages.iter() {
    ///     println!("Role: {:?}", message.role);
    /// }
    /// ```
    pub fn iter(&self) -> std::slice::Iter<'_, Message> {
        self.0.iter()
    }
}

impl From<Vec<Message>> for ChatMessages {
    fn from(value: Vec<Message>) -> Self {
        ChatMessages(value)
    }
}

impl From<&Vec<Message>> for ChatMessages {
    fn from(value: &Vec<Message>) -> Self {
        ChatMessages(value.to_owned())
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
