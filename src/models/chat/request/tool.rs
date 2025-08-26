use std::collections::HashMap;

use bon::bon;
use serde::{Deserialize, Serialize};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "function", rename_all = "snake_case")]
pub enum Tool {
    Function {
        name: String,
        description: Option<String>,
        parameters: Option<ParamType>,
    },
}

#[bon]
impl Tool {
    /// Initialize a builder to create a tool function definition.
    ///
    /// # Examples
    /// ```
    /// use orpheus::prelude::*;
    /// use serde_json::json;
    ///
    /// let target =  json!({
    ///     "type": "function",
    ///     "function": {
    ///         "name": "get_current_weather",
    ///         "description": "Get the current weather in a given location",
    ///         "parameters": {
    ///             "type": "object",
    ///             "properties": {
    ///                 "location": {
    ///                     "type": "string",
    ///                     "description": "The city and state, e.g. San Francisco, CA",
    ///                 },
    ///                 "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]},
    ///             },
    ///             "required": ["location"],
    ///         },
    ///     }
    /// });
    ///
    /// let tool = Tool::function("get_current_weather")
    ///         .description("Get the current weather in a given location")
    ///         .with_parameters(|p| {
    ///             p.property("location", Param::string().description("The city and state, e.g. San Francisco, CA").end())
    ///             .property("unit", Param::string().r#enum(["celsius", "fahrenheit"]).end())
    ///             .required(["location"])
    ///         })
    ///         .build();
    ///
    /// let tool_json = serde_json::to_value(&tool).unwrap();
    ///
    /// assert_eq!(target, tool_json);
    /// ```
    ///
    /// ## Name-Only Tool
    ///
    /// Some providers allow name-only tools; These can be useful as simple
    /// switches to let the model take actions.
    /// ```
    /// use orpheus::prelude::*;
    /// use serde_json::json;
    ///
    /// let target = json!({
    ///     "type": "function",
    ///     "function": {
    ///         "name": "my_tool"
    ///     }
    /// });
    ///
    /// let tool = Tool::function("my_tool").build();
    ///
    /// let tool_json = serde_json::to_value(&tool).unwrap();
    ///
    /// assert_eq!(target, tool_json);
    /// ```
    /// ## Empty Tool
    ///
    /// Some providers require that tools include parameters, even if they are empty.
    /// The `empty` method is a shortcut to build a Tool with and empty parameters object.
    ///
    /// ```
    /// use orpheus::prelude::*;
    /// use serde_json::json;
    ///
    /// let target = json!({
    ///     "type": "function",
    ///     "function": {
    ///         "name": "test_tool",
    ///         "parameters": {
    ///             "type": "object",
    ///             "properties": {}
    ///         }
    ///     }
    /// });
    ///
    /// let tool = Tool::function("test_tool").empty();
    ///
    /// let tool_json = serde_json::to_value(&tool).unwrap();
    ///
    /// assert_eq!(target, tool_json);
    /// ```
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParamType {
    Simple(Param),
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
        items: Box<ParamType>,
    },
    Object {
        description: Option<String>,
        properties: HashMap<String, ParamType>,
        required: Option<Vec<String>>,
        #[serde(rename = "additionalProperties")]
        additional_properties: Option<bool>,
    },
    Number {
        description: Option<String>,
    },
    Boolean {
        description: Option<String>,
    },
    Null,
}

impl Param {
    pub fn null() -> Self {
        Self::Null
    }

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

#[derive(Debug, Clone, Serialize)]
pub struct Tools(Vec<Tool>);

impl Tools {
    pub fn new(tools: Vec<Tool>) -> Self {
        Self(tools)
    }
}

impl From<Vec<Tool>> for Tools {
    fn from(tools: Vec<Tool>) -> Self {
        Self(tools)
    }
}

impl<const N: usize> From<[Tool; N]> for Tools {
    fn from(tools: [Tool; N]) -> Self {
        Self(tools.to_vec())
    }
}

pub trait Parameter {
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

#[macro_export]
macro_rules! anyof {
    ($($param:expr),* $(,)?) => {{
        use crate::models::chat::ParamType;

        let any_of: Vec<Param> = vec![$($param.into_param()),*];
        ParamType::Any { any_of }
    }};
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
}
