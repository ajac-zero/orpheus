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
        parameters: Option<Param>,
    },
}

#[bon]
impl Tool {
    /// Initialize a builder to create a tool function definition.
    ///
    /// # Examples
    /// ```
    /// use orpheus::{Tool, Param};
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
    /// use orpheus::Tool;
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
    /// use orpheus::Tool;
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
        parameters: Option<Param>,
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
    #[builder(on(String, into), finish_fn = end)]
    pub fn object(
        #[builder(field)] properties: HashMap<String, Self>,
        description: Option<String>,
        #[builder(with = |keys: impl IntoIterator<Item: Into<String>>| keys.into_iter().map(Into::into).collect())]
        required: Option<Vec<String>>,
    ) -> Self {
        Self::Object {
            description,
            properties,
            required,
        }
    }

    #[builder(on(String, into), finish_fn = end)]
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

    #[builder(on(String, into), finish_fn = end)]
    pub fn integer(description: Option<String>) -> Self {
        Self::Integer { description }
    }

    #[builder(on(String, into), finish_fn = end)]
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
