mod content;
mod message;
mod plugins;
mod provider;
mod reasoning;
mod tool;
mod usage;

pub use content::Content;
pub use message::{ChatMessages, Message, Role, ToolCall};
pub use plugins::{ParsingEngine, Plugin};
pub use provider::ProviderPreferences;
pub use reasoning::{ReasoningConfig, ReasoningEffort};
pub use tool::{Param, Tool};
pub use usage::UsageConfig;

#[cfg(test)]
mod test {
    use serde_json::{Value, from_value, json};

    use super::*;

    #[test]
    fn test_chat_message_simple_content() {
        let message = Message {
            role: Role::User,
            content: Content::Simple("Hello world!".to_string()),
            tool_calls: None,
            annotations: None,
        };

        let json = serde_json::to_string(&message).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();

        match deserialized.content {
            Content::Simple(text) => assert_eq!(text, "Hello world!"),
            _ => panic!("Expected simple content"),
        }
        assert!(matches!(deserialized.role, Role::User));
    }

    #[test]
    fn test_message_roles_serialization() {
        let roles = vec![
            Role::System,
            Role::Developer,
            Role::User,
            Role::Assistant,
            Role::Tool,
        ];

        for role in roles {
            let message = Message {
                role: role.clone(),
                content: Content::Simple("test".to_string()),
                tool_calls: None,
                annotations: None,
            };

            let json = serde_json::to_string(&message).unwrap();
            let deserialized: Message = serde_json::from_str(&json).unwrap();

            // Test that roles serialize/deserialize correctly
            match (&role, &deserialized.role) {
                (Role::System, Role::System) => (),
                (Role::Developer, Role::Developer) => (),
                (Role::User, Role::User) => (),
                (Role::Assistant, Role::Assistant) => (),
                (Role::Tool, Role::Tool) => (),
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

        let model = from_value::<Message>(data).unwrap();
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

        let model = from_value::<Message>(data).unwrap();
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

        let model = from_value::<Message>(data).unwrap();
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

        let model = from_value::<Message>(data).unwrap();
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

        let plugin = from_value::<Plugin>(payload).unwrap();
        println!("Web Plugin: {:?}", plugin);

        // assert that the plugin is of variant FileParser
        match plugin {
            Plugin::FileParser { pdf } => {
                assert!(matches!(pdf, ParsingEngine::PdfText));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_web_plugin_with_params_deserialize() {
        let payload = json!({"id": "web" });

        let plugin = from_value::<Plugin>(payload).unwrap();
        println!("Web Plugin: {:?}", plugin);

        // assert that the plugin is of variant FileParser
        match plugin {
            Plugin::Web {
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

        let plugin = from_value::<Plugin>(payload).unwrap();
        println!("Web Plugin: {:?}", plugin);

        // assert that the plugin is of variant FileParser
        match plugin {
            Plugin::Web {
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
                            .end(),
                    )
                    .property(
                        "unit",
                        Param::string().r#enum(["celsius", "fahrenheit"]).end(),
                    )
                    .required(["location"])
                    .end(),
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
                            .items(Param::string().end())
                            .end()
                    )
                    .required(["search_terms"])
                    .end()
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
                            .end(),
                    )
                    .property(
                        "unit",
                        Param::string().r#enum(["celsius", "fahrenheit"]).end(),
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
                            .items(Param::string().end())
                            .end()
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
