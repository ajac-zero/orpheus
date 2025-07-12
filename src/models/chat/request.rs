use std::collections::HashMap;

use bon::bon;
use serde::{Deserialize, Serialize};

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
#[serde(untagged)]
pub enum Content {
    Simple(String),
    Complex(Vec<Part>),
}

impl From<Vec<Part>> for Content {
    fn from(value: Vec<Part>) -> Self {
        Self::Complex(value)
    }
}

impl Content {
    pub fn simple(content: impl Into<String>) -> Self {
        Content::Simple(content.into())
    }

    pub fn with_image(self, url: impl Into<String>, detail: Option<String>) -> Self {
        let image_part = Part::image_url(url.into(), detail);
        self.add_part(image_part)
    }

    pub fn with_file(self, filename: impl Into<String>, data: impl Into<String>) -> Self {
        let file_part = Part::file(filename.into(), data.into());
        self.add_part(file_part)
    }

    /// Consumes the current content and creates a new content with the appended part.
    /// 1. `Self::Simple` variant is transformed into a complex variant with the original text prepended as a "text" part.
    /// 2. `Self::Complex` variant is modified by appending the new part to the existing parts vector.
    fn add_part(self, part: Part) -> Self {
        let new_parts = match self {
            Self::Simple(string) => vec![Part::text(string), part],
            Self::Complex(mut parts) => {
                parts.push(part);
                parts
            }
        };
        Content::Complex(new_parts)
    }

    pub fn to_string(&self) -> String {
        match self {
            Content::Simple(s) => s.clone(),
            Content::Complex(_) => todo!(),
        }
    }
}

impl From<String> for Content {
    fn from(string: String) -> Self {
        Content::Simple(string)
    }
}

impl<'a> From<&'a str> for Content {
    fn from(s: &'a str) -> Self {
        Content::Simple(s.to_string())
    }
}

impl std::fmt::Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::Simple(s) => write!(f, "{}", s),
            Content::Complex(v) => v.iter().try_for_each(|p| write!(f, "{}", p)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageUrl {
    url: String,
    detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct File {
    filename: String,
    file_data: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Part {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
    File { file: File },
}

impl Part {
    pub fn text(string: String) -> Self {
        Self::Text { text: string }
    }

    pub fn image_url(url: String, detail: Option<String>) -> Self {
        Self::ImageUrl {
            image_url: ImageUrl { url, detail },
        }
    }

    pub fn file(filename: String, data: String) -> Self {
        Self::File {
            file: File {
                filename,
                file_data: data,
            },
        }
    }
}

impl std::fmt::Display for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Part::Text { text } => write!(f, "{}", text),
            Part::ImageUrl { image_url } => write!(f, "{}", format!("[Url: {}]", image_url.url)),
            Part::File { file } => write!(f, "{}", format!("[File: {}]", file.filename)),
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
pub struct ProviderPreferences {
    /// Sort preference (e.g., price, throughput).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningConfig {
    /// OpenAI-style reasoning effort setting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<ReasoningEffort>,

    /// Non-OpenAI-style reasoning effort setting. Cannot be used simultaneously with effort.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,

    /// Whether to exclude reasoning from the response. Defaults to false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageConfig {
    /// Whether to include usage information in the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "engine", rename_all = "kebab-case")]
pub enum ParsingEngine {
    PdfText,
    MistralOcr,
    Native,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "id", rename_all = "kebab-case")]
pub enum Plugins {
    FileParser {
        pdf: ParsingEngine,
    },
    Web {
        max_results: Option<u64>,
        search_prompt: Option<String>,
    },
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

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "function", rename_all = "snake_case")]
pub enum Tool {
    Function {
        name: String,
        description: Option<String>,
        parameters: Option<Param>,
    },
}

#[bon]
impl Tool {
    #[builder(on(String, into), finish_fn = build)]
    pub fn function(
        #[builder(start_fn)] name: String,
        description: Option<String>,
        parameters: Option<Param>,
    ) -> Self {
        Self::Function {
            name,
            description,
            parameters,
        }
    }
}

impl<S: tool_function_builder::State> ToolFunctionBuilder<S> {
    pub fn with_parameters<F, C>(
        self,
        build: F,
    ) -> ToolFunctionBuilder<tool_function_builder::SetParameters<S>>
    where
        S::Parameters: tool_function_builder::IsUnset,
        F: FnOnce(ParamObjectBuilder<param_object_builder::Empty>) -> ParamObjectBuilder<C>,
        C: param_object_builder::IsComplete,
    {
        let builder = Param::object();
        let parameters = build(builder).call();
        self.parameters(parameters)
    }
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Param {
    Integer {
        description: Option<String>,
    },
    r#String {
        description: Option<String>,
        r#enum: Option<Vec<String>>,
    },
    Array {
        description: Option<String>,
        items: Box<Param>,
    },
    Object {
        description: Option<String>,
        properties: HashMap<String, Param>,
        required: Option<Vec<String>>,
    },
}

#[bon]
impl Param {
    #[builder]
    pub fn object(
        #[builder(field)] properties: HashMap<String, Self>,
        #[builder(into)] description: Option<String>,
        #[builder(with = |keys: impl IntoIterator<Item: Into<String>>| keys.into_iter().map(Into::into).collect())]
        required: Option<Vec<String>>,
    ) -> Self {
        Self::Object {
            description,
            properties,
            required,
        }
    }

    #[builder(on(String, into))]
    pub fn string(
        description: Option<String>,
        #[builder(with = |keys: impl IntoIterator<Item: Into<String>>| keys.into_iter().map(Into::into).collect())]
        r#enum: Option<Vec<String>>,
    ) -> Self {
        Self::String {
            description,
            r#enum,
        }
    }

    #[builder(on(String, into))]
    pub fn integer(description: Option<String>) -> Self {
        Self::Integer { description }
    }

    #[builder(on(String, into))]
    pub fn array(description: Option<String>, items: Param) -> Self {
        Self::Array {
            description,
            items: Box::new(items),
        }
    }
}

impl<S: param_object_builder::State> ParamObjectBuilder<S> {
    pub fn property(mut self, key: impl Into<String>, value: Param) -> Self {
        self.properties.insert(key.into(), value);
        self
    }

    pub fn properties(mut self, properties: HashMap<String, Param>) -> Self {
        self.properties = properties;
        self
    }
}

#[cfg(test)]
mod test {
    use serde_json::{Value, from_value, json};

    use super::*;

    #[test]
    fn test_chat_message_simple_content() {
        let message = ChatMessage {
            role: MessageRole::User,
            content: Content::Simple("Hello world!".to_string()),
            tool_calls: None,
            annotations: None,
        };

        let json = serde_json::to_string(&message).unwrap();
        let deserialized: ChatMessage = serde_json::from_str(&json).unwrap();

        match deserialized.content {
            Content::Simple(text) => assert_eq!(text, "Hello world!"),
            _ => panic!("Expected simple content"),
        }
        assert!(matches!(deserialized.role, MessageRole::User));
    }

    #[test]
    fn test_message_roles_serialization() {
        let roles = vec![
            MessageRole::System,
            MessageRole::Developer,
            MessageRole::User,
            MessageRole::Assistant,
            MessageRole::Tool,
        ];

        for role in roles {
            let message = ChatMessage {
                role: role.clone(),
                content: Content::Simple("test".to_string()),
                tool_calls: None,
                annotations: None,
            };

            let json = serde_json::to_string(&message).unwrap();
            let deserialized: ChatMessage = serde_json::from_str(&json).unwrap();

            // Test that roles serialize/deserialize correctly
            match (&role, &deserialized.role) {
                (MessageRole::System, MessageRole::System) => (),
                (MessageRole::Developer, MessageRole::Developer) => (),
                (MessageRole::User, MessageRole::User) => (),
                (MessageRole::Assistant, MessageRole::Assistant) => (),
                (MessageRole::Tool, MessageRole::Tool) => (),
                _ => panic!("Role mismatch: {:?} != {:?}", role, deserialized.role),
            }
        }
    }

    #[test]
    fn test_reasoning_effort_serialization() {
        let efforts = vec![
            ReasoningEffort::High,
            ReasoningEffort::Medium,
            ReasoningEffort::Low,
        ];

        for effort in efforts {
            let config = ReasoningConfig {
                effort: Some(effort.clone()),
                max_tokens: None,
                exclude: None,
            };

            let json = serde_json::to_string(&config).unwrap();
            let deserialized: ReasoningConfig = serde_json::from_str(&json).unwrap();

            assert!(deserialized.effort.is_some());
            match (&effort, deserialized.effort.as_ref().unwrap()) {
                (ReasoningEffort::High, ReasoningEffort::High) => (),
                (ReasoningEffort::Medium, ReasoningEffort::Medium) => (),
                (ReasoningEffort::Low, ReasoningEffort::Low) => (),
                _ => panic!("Effort mismatch"),
            }
        }
    }

    #[test]
    fn test_simple_message_deserialization() {
        let data = json!(                {
                    "role": "user",
                    "content": "hello!"
        });

        let model = from_value::<ChatMessage>(data).unwrap();
        println!("Chat Message: {:?}", model);
    }

    #[test]
    fn test_text_type_message_deserialization() {
        let data = json!(                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": "hii!"
                        }
                    ]
        });

        let model = from_value::<ChatMessage>(data).unwrap();
        println!("Chat Message: {:?}", model);
    }

    #[test]
    fn test_image_type_message_deserialization() {
        let data = json!(                {
                    "role": "user",
                    "content": [
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": "https://upload.wikimedia.org/wikipedia/commons/thumb/d/dd/Gfp-wisconsin-madison-the-nature-boardwalk.jpg/2560px-Gfp-wisconsin-madison-the-nature-boardwalk.jpg"
                            }
                        }
                    ]
        });

        let model = from_value::<ChatMessage>(data).unwrap();
        println!("Chat Message: {:?}", model);
    }

    #[test]
    fn test_complex_request_deserialization() {
        let data = json!({
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": "What's in this image?"
                        },
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": "https://upload.wikimedia.org/wikipedia/commons/thumb/d/dd/Gfp-wisconsin-madison-the-nature-boardwalk.jpg/2560px-Gfp-wisconsin-madison-the-nature-boardwalk.jpg"
                            }
                        }
                    ]
        });

        let model = from_value::<ChatMessage>(data).unwrap();
        println!("Complex Chat Message: {:?}", model);
    }

    #[test]
    fn test_file_parser_plugin_deserialize() {
        let payload = json!({
              "id": "file-parser",
              "pdf": {
                "engine": "pdf-text", // or 'mistral-ocr' or 'native'
              },
        });

        let plugin = from_value::<Plugins>(payload).unwrap();
        println!("Web Plugin: {:?}", plugin);

        // assert that the plugin is of variant FileParser
        match plugin {
            Plugins::FileParser { pdf } => {
                assert!(matches!(pdf, ParsingEngine::PdfText));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_web_plugin_with_params_deserialize() {
        let payload = json!({"id": "web" });

        let plugin = from_value::<Plugins>(payload).unwrap();
        println!("Web Plugin: {:?}", plugin);

        // assert that the plugin is of variant FileParser
        match plugin {
            Plugins::Web {
                max_results,
                search_prompt,
            } => {
                assert!(max_results.is_none());
                assert!(search_prompt.is_none());
            }
            _ => unreachable!(),
        }

        let payload =
            json!({"id": "web", "max_results": 10, "search_prompt": "Some relevant web results:" });

        let plugin = from_value::<Plugins>(payload).unwrap();
        println!("Web Plugin: {:?}", plugin);

        // assert that the plugin is of variant FileParser
        match plugin {
            Plugins::Web {
                max_results,
                search_prompt,
            } => {
                assert!(max_results == Some(10));
                assert!(search_prompt == Some("Some relevant web results:".to_string()));
            }
            _ => unreachable!(),
        }
    }

    fn get_current_weather_json() -> Value {
        json!({
          "type": "function",
          "function": {
            "name": "get_current_weather",
            "description": "Get the current weather in a given location",
            "parameters": {
              "type": "object",
              "properties": {
                "location": {
                  "type": "string",
                  "description": "The city and state, e.g. San Francisco, CA",
                },
                "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]},
              },
              "required": ["location"],
            },
          }
        })
    }

    fn search_gutenberg_books_json() -> Value {
        json!({
          "type": "function",
          "function": {
            "name": "search_gutenberg_books",
            "description": "Search for books in the Project Gutenberg library based on specified search terms",
            "parameters": {
              "type": "object",
              "properties": {
                "search_terms": {
                  "type": "array",
                  "items": {
                    "type": "string"
                  },
                  "description": "List of search terms to find books in the Gutenberg library (e.g. ['dickens', 'great'] to search for books by Dickens with 'great' in the title)"
                }
              },
              "required": ["search_terms"]
            }
          }
        })
    }

    #[test]
    fn test_deserialize_tool_call() {
        let get_current_weather = get_current_weather_json();

        let function: Tool = serde_json::from_value(get_current_weather).unwrap();
        println!("Function 1: {:?}\n", function);

        let search_gutenberg_books = search_gutenberg_books_json();

        let function: Tool = serde_json::from_value(search_gutenberg_books).unwrap();
        println!("Function 2: {:?}\n", function);
    }

    #[test]
    fn test_serialize_tool_call() {
        let tool = Tool::function("get_current_weather")
            .description("Get the current weather in a given location")
            .parameters(
                Param::object()
                    .property(
                        "location",
                        Param::string()
                            .description("The city and state, e.g. San Francisco, CA")
                            .call(),
                    )
                    .property(
                        "unit",
                        Param::string().r#enum(["celsius", "fahrenheit"]).call(),
                    )
                    .required(["location"])
                    .call(),
            )
            .build();

        let function = serde_json::to_value(&tool).unwrap();

        let payload = get_current_weather_json();

        assert_eq!(function, payload);

        let tool = Tool::function("search_gutenberg_books")
            .description("Search for books in the Project Gutenberg library based on specified search terms")
            .parameters(
                Param::object()
                    .property(
                        "search_terms",
                        Param::array()
                            .description("List of search terms to find books in the Gutenberg library (e.g. ['dickens', 'great'] to search for books by Dickens with 'great' in the title)")
                            .items(Param::string().call())
                            .call()
                    )
                    .required(["search_terms"])
                    .call()
            )
            .build();

        let function = serde_json::to_value(&tool).unwrap();

        let payload = search_gutenberg_books_json();

        assert_eq!(function, payload);
    }

    #[test]
    fn test_serialize_tool_call_with_closure() {
        // Test the new simplified API using closure
        let tool = Tool::function("get_current_weather")
            .description("Get the current weather in a given location")
            .with_parameters(|params| {
                params
                    .property(
                        "location",
                        Param::string()
                            .description("The city and state, e.g. San Francisco, CA")
                            .call(),
                    )
                    .property(
                        "unit",
                        Param::string().r#enum(["celsius", "fahrenheit"]).call(),
                    )
                    .required(["location"])
            })
            .build();

        let function = serde_json::to_value(&tool).unwrap();

        let payload = get_current_weather_json();

        assert_eq!(function, payload);

        let tool = Tool::function("search_gutenberg_books")
            .description("Search for books in the Project Gutenberg library based on specified search terms")
            .with_parameters(|params| {
                    params.property(
                        "search_terms",
                        Param::array()
                            .description("List of search terms to find books in the Gutenberg library (e.g. ['dickens', 'great'] to search for books by Dickens with 'great' in the title)")
                            .items(Param::string().call())
                            .call()
                    )
                    .required(["search_terms"])
                }
            )
            .build();

        let function = serde_json::to_value(&tool).unwrap();

        let payload = search_gutenberg_books_json();

        assert_eq!(function, payload);
    }
}
