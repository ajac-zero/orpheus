use std::collections::HashMap;

use bon::bon;
use serde::{Deserialize, Serialize};

/// Represents a tool that can be called by the language model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "function", rename_all = "snake_case")]
pub enum Tool {
    /// A function tool that the model can call with structured parameters.
    Function {
        /// The name of the function (must be unique within a tool set)
        name: String,
        /// Optional description explaining what the function does
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// Optional parameter schema defining the function's input structure
        #[serde(skip_serializing_if = "Option::is_none")]
        parameters: Option<ParamType>,
    },
}

#[bon]
impl Tool {
    #[builder(on(String, into), finish_fn = build)]
    pub fn function(
        #[builder(start_fn)] name: String,
        description: Option<String>,
        #[builder(into)] parameters: Option<ParamType>,
    ) -> Self {
        Self::Function {
            name,
            description,
            parameters,
        }
    }
}

impl<S: tool_function_builder::State> ToolFunctionBuilder<S>
where
    S::Parameters: tool_function_builder::IsUnset,
{
    pub fn empty(self) -> Tool {
        self.with_parameters(|p| p).build()
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
        let parameters = build(builder).end();
        self.parameters(parameters)
    }
}

/// Wraper type representing either a single parameter or a union type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParamType {
    /// A single parameter type
    Simple(Param),
    /// A union type that can match any of the specified parameter types
    Any {
        #[serde(rename = "anyOf")]
        any_of: Vec<Param>,
    },
}

impl From<Param> for ParamType {
    fn from(param: Param) -> Self {
        ParamType::Simple(param)
    }
}

/// Represents a JSON Schema parameter definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Param {
    /// Integer parameter (whole numbers only)
    Integer {
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    /// String parameter with optional enumeration constraints
    r#String {
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        r#enum: Option<Vec<String>>,
    },
    /// Array parameter with specified item type
    Array {
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// The type definition for array items
        items: Box<ParamType>,
    },
    /// Object parameter with properties and constraints
    Object {
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// Map of property names to their parameter definitions
        properties: HashMap<String, ParamType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        required: Option<Vec<String>>,
        #[serde(rename = "additionalProperties", skip_serializing_if = "Option::is_none")]
        additional_properties: Option<bool>,
    },
    /// Number parameter (floating point numbers)
    Number {
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    /// Boolean parameter (true/false values)
    Boolean {
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    /// Null parameter (represents JSON null)
    Null,
}

impl Param {
    /// Creates a null parameter.
    pub fn null() -> Self {
        Self::Null
    }

    /// Identity function that returns the parameter unchanged.
    pub fn into_param(self) -> Self {
        self
    }
}

#[bon]
impl Param {
    #[builder(finish_fn = end, state_mod(vis = "pub(crate)"))]
    pub fn object(
        #[builder(field)] properties: HashMap<String, ParamType>,
        #[builder(into)] description: Option<String>,
        #[builder(with = |keys: impl IntoIterator<Item: Into<String>>| keys.into_iter().map(Into::into).collect())]
        required: Option<Vec<String>>,
        additional_properties: Option<bool>,
    ) -> Self {
        Self::Object {
            description,
            properties,
            required,
            additional_properties,
        }
    }

    #[builder(finish_fn = end)]
    pub fn string(
        #[builder(into)] description: Option<String>,
        #[builder(with = |keys: impl IntoIterator<Item: Into<String>>| keys.into_iter().map(Into::into).collect())]
        enums: Option<Vec<String>>,
    ) -> Self {
        Self::String {
            description,
            r#enum: enums,
        }
    }

    #[builder(finish_fn = end)]
    pub fn integer(#[builder(into)] description: Option<String>) -> Self {
        Self::Integer { description }
    }

    #[builder(finish_fn = end)]
    pub fn array(
        #[builder(into)] description: Option<String>,
        #[builder(into)] items: ParamType,
    ) -> Self {
        Self::Array {
            description,
            items: Box::new(items),
        }
    }

    #[builder(finish_fn = end)]
    pub fn number(#[builder(into)] description: Option<String>) -> Self {
        Self::Number { description }
    }

    #[builder(finish_fn = end)]
    pub fn boolean(#[builder(into)] description: Option<String>) -> Self {
        Self::Boolean { description }
    }
}

impl<S: param_object_builder::State> ParamObjectBuilder<S> {
    pub fn property(mut self, key: impl Into<String>, value: impl Into<ParamType>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    pub fn properties(mut self, properties: HashMap<String, ParamType>) -> Self {
        self.properties = properties;
        self
    }
}

/// Trait for types that can be converted into a Param.
pub trait Parameter {
    /// Converts this type into a Param.
    fn into_param(self) -> Param;
}

impl<T: Parameter> From<T> for ParamType {
    fn from(value: T) -> Self {
        value.into_param().into()
    }
}

macro_rules! impl_parameter_for_builder {
    ($builder:ident, $state_mod:ident) => {
        impl<S: $state_mod::State> Parameter for $builder<S>
        where
            S: $state_mod::IsComplete,
        {
            fn into_param(self) -> Param {
                self.end()
            }
        }
    };
}

impl_parameter_for_builder!(ParamStringBuilder, param_string_builder);
impl_parameter_for_builder!(ParamObjectBuilder, param_object_builder);
impl_parameter_for_builder!(ParamArrayBuilder, param_array_builder);
impl_parameter_for_builder!(ParamNumberBuilder, param_number_builder);
impl_parameter_for_builder!(ParamBooleanBuilder, param_boolean_builder);
impl_parameter_for_builder!(ParamIntegerBuilder, param_integer_builder);

/// Creates a union type (anyOf) parameter from multiple parameter types.
#[macro_export]
macro_rules! anyof {
    ($($param:expr),* $(,)?) => {{
        use $crate::models::tool::{ParamType, Parameter};

        let any_of: Vec<Param> = vec![$($param.into_param()),*];
        ParamType::Any { any_of }
    }};
}

impl From<Tool> for open_responses::FunctionToolParam {
    fn from(tool: Tool) -> Self {
        match tool {
            Tool::Function {
                name,
                description,
                parameters,
            } => {
                let params_value =
                    parameters.map(|p| serde_json::to_value(p).unwrap_or_default());
                open_responses::FunctionToolParam {
                    type_: "function".into(),
                    name,
                    description,
                    parameters: params_value,
                    strict: None,
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_shorthand_tool_dev() {
        let target = json!({
            "type": "object",
            "properties": {
                "location":{
                    "type": "object",
                    "description": "Coordinates of the location",
                    "properties": {
                        "latitude": { "type": "string" },
                        "longitude": { "type": "string" }
                    },
                    "required": ["latitude", "longitude"]
                },
                "units": {
                    "type": "string",
                    "description": "Units the temperature will be returned in.",
                    "enum": ["celsius", "fahrenheit"]
                }
            },
            "required": ["location", "units"]
        });

        let param = Param::object()
            .property(
                "location",
                Param::object()
                    .description("Coordinates of the location")
                    .property("latitude", Param::string())
                    .property("longitude", Param::string())
                    .required(["latitude", "longitude"]),
            )
            .property(
                "units",
                Param::string()
                    .description("Units the temperature will be returned in.")
                    .enums(["celsius", "fahrenheit"]),
            )
            .required(["location", "units"])
            .end();

        let value = serde_json::to_value(param).unwrap();

        assert_eq!(target, value);
    }

    #[test]
    fn deser_vec_tool_def() {
        let target = json!({
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "base_branch": {
                        "anyOf": [{"type": "string"}, {"type": "null"}]
                    },
                    "branch_name": {"type": "string"},
                    "repo_path": {"type": "string"}
                }
            }
        });

        let param = Param::array()
            .items(
                Param::object()
                    .property("base_branch", anyof![Param::string(), Param::null()])
                    .property("branch_name", Param::string())
                    .property("repo_path", Param::string()),
            )
            .end();
        let param_value = serde_json::to_value(param).unwrap();

        assert_eq!(target, param_value);
    }

    #[test]
    fn test_charades_tools() {
        let target = json!({
            "type": "function",
            "function": {
                "name": "player_win",
                "parameters": {
                    "type": "object",
                    "properties": {}
                }
            }
        });

        let tool = Tool::function("player_win").empty();
        let value = serde_json::to_value(tool).unwrap();

        assert_eq!(target, value);

        let target = json!({
            "type": "function",
            "function": {
                "name": "game_over",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "answer": { "type": "string" }
                    }
                }
            }
        });

        let tool = Tool::function("game_over")
            .with_parameters(|p| p.property("answer", Param::string()))
            .build();
        let value = serde_json::to_value(tool).unwrap();

        assert_eq!(target, value);
    }

    #[test]
    fn test_serialize_tool_call() {
        let tool = Tool::function("get_current_weather")
            .description("Get the current weather in a given location")
            .parameters(
                Param::object()
                    .property(
                        "location",
                        Param::string().description("The city and state, e.g. San Francisco, CA"),
                    )
                    .property(
                        "unit",
                        Param::string().enums(["celsius", "fahrenheit"]).end(),
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
        let tool = Tool::function("get_current_weather")
            .description("Get the current weather in a given location")
            .with_parameters(|params| {
                params
                    .property(
                        "location",
                        Param::string().description("The city and state, e.g. San Francisco, CA"),
                    )
                    .property("unit", Param::string().enums(["celsius", "fahrenheit"]))
                    .required(["location"])
            })
            .build();

        let function = serde_json::to_value(&tool).unwrap();

        let payload = get_current_weather_json();

        assert_eq!(function, payload);

        let tool = Tool::function("search_gutenberg_books")
            .description(
                "Search for books in the Project Gutenberg library based on specified search terms",
            )
            .with_parameters(|params| {
                params
                    .property(
                        "search_terms",
                        Param::array()
                            .description("List of search terms to find books in the Gutenberg library (e.g. ['dickens', 'great'] to search for books by Dickens with 'great' in the title)")
                            .items(Param::string().end())
                            .end(),
                    )
                    .required(["search_terms"])
            })
            .build();

        let function = serde_json::to_value(&tool).unwrap();

        let payload = search_gutenberg_books_json();

        assert_eq!(function, payload);
    }

    fn get_current_weather_json() -> serde_json::Value {
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

    fn search_gutenberg_books_json() -> serde_json::Value {
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
    fn test_tool_to_function_tool_param() {
        let tool = Tool::function("get_weather")
            .description("Get weather")
            .with_parameters(|p| p.property("city", Param::string()).required(["city"]))
            .build();

        let param: open_responses::FunctionToolParam = tool.into();
        assert_eq!(param.type_, "function");
        assert_eq!(param.name, "get_weather");
        assert_eq!(param.description, Some("Get weather".to_string()));
        assert!(param.parameters.is_some());
    }
}
