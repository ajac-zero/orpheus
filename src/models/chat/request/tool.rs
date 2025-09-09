use std::collections::HashMap;

use bon::bon;
use serde::{Deserialize, Serialize};

/// Tool definition for function calling capabilities.
///
/// This module provides the core types for defining tools (functions) that
/// language models can call during conversations. Tools allow models to
/// interact with external systems, retrieve data, or perform actions
/// beyond text generation.

/// Represents a tool that can be called by the language model.
///
/// Currently supports function-type tools, which allow the model to call
/// external functions with structured parameters. The model will decide
/// when and how to use these tools based on the conversation context.
///
/// # Examples
///
/// ```rust
/// use orpheus::prelude::*;
///
/// // Simple tool without parameters
/// let tool = Tool::function("get_time").build();
///
/// // Tool with parameters
/// let tool = Tool::function("get_weather")
///     .description("Get current weather for a location")
///     .with_parameters(|params| {
///         params
///             .property("location", Param::string().description("City name"))
///             .property("units", Param::string().enums(["celsius", "fahrenheit"]))
///             .required(["location"])
///     })
///     .build();
/// ```
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "function", rename_all = "snake_case")]
pub enum Tool {
    /// A function tool that the model can call with structured parameters.
    Function {
        /// The name of the function (must be unique within a tool set)
        name: String,
        /// Optional description explaining what the function does
        description: Option<String>,
        /// Optional parameter schema defining the function's input structure
        parameters: Option<ParamType>,
    },
}

#[bon]
impl Tool {
    /// Creates a builder for defining a function tool.
    ///
    /// This is the primary entry point for creating tools that the language model
    /// can call. The builder pattern allows you to incrementally specify the
    /// function name, description, and parameter schema.
    ///
    /// # Parameters
    ///
    /// * `name` - The function name (must be a valid identifier)
    /// * `description` - Optional human-readable description of what the function does
    /// * `parameters` - Optional parameter schema defining the function's inputs
    ///
    /// # Examples
    ///
    /// ## Basic Function Tool
    /// ```rust
    /// use orpheus::prelude::*;
    /// use serde_json::json;
    ///
    /// let target = json!({
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
    ///     .description("Get the current weather in a given location")
    ///     .with_parameters(|params| {
    ///         params
    ///             .property("location", Param::string()
    ///                 .description("The city and state, e.g. San Francisco, CA"))
    ///             .property("unit", Param::string().enums(["celsius", "fahrenheit"]))
    ///             .required(["location"])
    ///     })
    ///     .build();
    ///
    /// let tool_json = serde_json::to_value(&tool).unwrap();
    /// assert_eq!(target, tool_json);
    /// ```
    ///
    /// ## Simple Name-Only Tool
    ///
    /// Some providers allow tools without parameters, useful for simple actions:
    ///
    /// ```rust
    /// use orpheus::prelude::*;
    /// use serde_json::json;
    ///
    /// let target = json!({
    ///     "type": "function",
    ///     "function": {
    ///         "name": "end_conversation"
    ///     }
    /// });
    ///
    /// let tool = Tool::function("end_conversation").build();
    /// let tool_json = serde_json::to_value(&tool).unwrap();
    /// assert_eq!(target, tool_json);
    /// ```
    ///
    /// ## Empty Parameters Tool
    ///
    /// Some providers require explicit empty parameter objects:
    ///
    /// ```rust
    /// use orpheus::prelude::*;
    /// use serde_json::json;
    ///
    /// let target = json!({
    ///     "type": "function",
    ///     "function": {
    ///         "name": "get_current_time",
    ///         "parameters": {
    ///             "type": "object",
    ///             "properties": {}
    ///         }
    ///     }
    /// });
    ///
    /// let tool = Tool::function("get_current_time").empty();
    /// let tool_json = serde_json::to_value(&tool).unwrap();
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
    /// Creates a tool with an empty parameters object.
    ///
    /// This is a convenience method for tools that don't need parameters
    /// but require an explicit empty parameters object for compatibility
    /// with certain providers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orpheus::prelude::*;
    ///
    /// let tool = Tool::function("ping").empty();
    /// // Equivalent to:
    /// // let tool = Tool::function("ping")
    /// //     .with_parameters(|params| params)
    /// //     .build();
    /// ```
    pub fn empty(self) -> Tool {
        self.with_parameters(|p| p).build()
    }
}

impl<S: tool_function_builder::State> ToolFunctionBuilder<S> {
    /// Defines the function's parameters using a builder closure.
    ///
    /// This method provides an ergonomic way to define the parameter schema
    /// for a function tool. The closure receives a `ParamObjectBuilder` that
    /// can be used to define properties, types, requirements, and other
    /// constraints for the function's input parameters.
    ///
    /// # Parameters
    ///
    /// * `build` - A closure that configures the parameter schema using the builder API
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orpheus::prelude::*;
    ///
    /// let tool = Tool::function("search_database")
    ///     .description("Search for records in the database")
    ///     .with_parameters(|params| {
    ///         params
    ///             .property("query", Param::string().description("Search query"))
    ///             .property("limit", Param::integer().description("Maximum results"))
    ///             .property("filters", Param::object()
    ///                 .property("category", Param::string())
    ///                 .property("date_range", Param::array()
    ///                     .items(Param::string().end())
    ///                     .end())
    ///                 .end())
    ///             .required(["query"])
    ///     })
    ///     .build();
    /// ```
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

/// Represents a parameter type that can be either a single parameter or a union type.
///
/// This enum allows for flexible parameter definitions that can accept multiple
/// possible types. The `Simple` variant represents a single parameter type,
/// while the `Any` variant represents a union type (anyOf in JSON Schema).
///
/// # Examples
///
/// ```rust
/// use orpheus::{anyof, models::chat::{Param, ParamType}};
///
/// // Simple parameter type
/// let simple_param: ParamType = Param::string().into();
///
/// // Union type using the anyof! macro
/// let union_param = anyof![
///     Param::string(),
///     Param::number(),
///     Param::null()
/// ];
/// ```
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
///
/// This enum covers all the basic JSON Schema types and their associated
/// properties. Each variant can include a description and type-specific
/// constraints like enums for strings or item types for arrays.
///
/// # Examples
///
/// ```rust
/// use orpheus::prelude::*;
/// use std::collections::HashMap;
///
/// // Simple string parameter
/// let name_param = Param::string().description("User's full name").end();
///
/// // Integer with constraints would be defined via builder
/// let age_param = Param::integer().description("Age in years").end();
///
/// // Complex object parameter
/// let user_param = Param::object()
///     .property("name", Param::string())
///     .property("age", Param::integer())
///     .required(["name"])
///     .end();
/// ```
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Param {
    /// Integer parameter (whole numbers only)
    Integer {
        /// Optional description explaining the parameter's purpose
        description: Option<String>,
    },
    /// String parameter with optional enumeration constraints
    r#String {
        /// Optional description explaining the parameter's purpose
        description: Option<String>,
        /// Optional list of allowed string values (enum constraint)
        r#enum: Option<Vec<String>>,
    },
    /// Array parameter with specified item type
    Array {
        /// Optional description explaining the parameter's purpose
        description: Option<String>,
        /// The type definition for array items
        items: Box<ParamType>,
    },
    /// Object parameter with properties and constraints
    Object {
        /// Optional description explaining the parameter's purpose
        description: Option<String>,
        /// Map of property names to their parameter definitions
        properties: HashMap<String, ParamType>,
        /// Optional list of required property names
        required: Option<Vec<String>>,
        /// Whether additional properties beyond those defined are allowed
        #[serde(rename = "additionalProperties")]
        additional_properties: Option<bool>,
    },
    /// Number parameter (floating point numbers)
    Number {
        /// Optional description explaining the parameter's purpose
        description: Option<String>,
    },
    /// Boolean parameter (true/false values)
    Boolean {
        /// Optional description explaining the parameter's purpose
        description: Option<String>,
    },
    /// Null parameter (represents JSON null)
    Null,
}

impl Param {
    /// Creates a null parameter.
    ///
    /// Represents a JSON null value in the schema. Useful for optional
    /// fields or union types that can be null.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orpheus::{anyof, models::chat::Param};
    ///
    /// // Simple null parameter
    /// let null_param = Param::null();
    ///
    /// // Union type that can be string or null
    /// let optional_string = anyof![
    ///     Param::string(),
    ///     Param::null()
    /// ];
    /// ```
    pub fn null() -> Self {
        Self::Null
    }

    /// Identity function that returns the parameter unchanged.
    ///
    /// This method exists to satisfy trait requirements and provide
    /// a consistent interface for parameter conversion.
    pub fn into_param(self) -> Self {
        self
    }
}

#[bon]
impl Param {
    /// Creates a builder for an object parameter.
    ///
    /// Object parameters represent JSON objects with defined properties,
    /// requirements, and constraints. This is the most complex parameter
    /// type and is commonly used for structured data.
    ///
    /// # Parameters
    ///
    /// * `properties` - Map of property names to their parameter definitions
    /// * `description` - Optional description of the object's purpose
    /// * `required` - Optional list of required property names
    /// * `additional_properties` - Whether extra properties are allowed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orpheus::prelude::*;
    ///
    /// // Simple object with string properties
    /// let person = Param::object()
    ///     .description("Person information")
    ///     .property("name", Param::string().description("Full name"))
    ///     .property("email", Param::string().description("Email address"))
    ///     .required(["name"])
    ///     .additional_properties(false)
    ///     .end();
    ///
    /// // Nested object structure
    /// let address = Param::object()
    ///     .property("street", Param::string())
    ///     .property("city", Param::string())
    ///     .property("coordinates", Param::object()
    ///         .property("lat", Param::number())
    ///         .property("lng", Param::number())
    ///         .required(["lat", "lng"])
    ///         .end())
    ///     .required(["street", "city"])
    ///     .end();
    /// ```
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

    /// Creates a builder for a string parameter.
    ///
    /// String parameters represent text values and can optionally be
    /// constrained to a specific set of allowed values (enums).
    ///
    /// # Parameters
    ///
    /// * `description` - Optional description of the string's purpose
    /// * `enums` - Optional list of allowed string values
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orpheus::prelude::*;
    ///
    /// // Simple string parameter
    /// let name = Param::string()
    ///     .description("User's full name")
    ///     .end();
    ///
    /// // String with enum constraints
    /// let status = Param::string()
    ///     .description("Current status")
    ///     .enums(["active", "inactive", "pending"])
    ///     .end();
    ///
    /// // String for email addresses
    /// let email = Param::string()
    ///     .description("Valid email address")
    ///     .end();
    /// ```
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

    /// Creates a builder for an integer parameter.
    ///
    /// Integer parameters represent whole numbers (no decimal places).
    /// Use this for counts, IDs, ages, and other discrete numeric values.
    ///
    /// # Parameters
    ///
    /// * `description` - Optional description of the integer's purpose
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orpheus::prelude::*;
    ///
    /// // Age parameter
    /// let age = Param::integer()
    ///     .description("Age in years")
    ///     .end();
    ///
    /// // Count parameter
    /// let count = Param::integer()
    ///     .description("Number of items to retrieve")
    ///     .end();
    ///
    /// // ID parameter
    /// let user_id = Param::integer()
    ///     .description("Unique user identifier")
    ///     .end();
    /// ```
    #[builder(finish_fn = end)]
    pub fn integer(#[builder(into)] description: Option<String>) -> Self {
        Self::Integer { description }
    }

    /// Creates a builder for an array parameter.
    ///
    /// Array parameters represent lists of items where all items conform
    /// to the same schema. The item type can be any valid parameter type,
    /// including complex objects or other arrays.
    ///
    /// # Parameters
    ///
    /// * `description` - Optional description of the array's purpose
    /// * `items` - Parameter definition for the array's item type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orpheus::prelude::*;
    ///
    /// // Array of strings
    /// let tags = Param::array()
    ///     .description("List of tags")
    ///     .items(Param::string().end())
    ///     .end();
    ///
    /// // Array of objects
    /// let users = Param::array()
    ///     .description("List of users")
    ///     .items(Param::object()
    ///         .property("name", Param::string())
    ///         .property("email", Param::string())
    ///         .required(["name", "email"])
    ///         .end())
    ///     .end();
    ///
    /// // Nested array (array of arrays)
    /// let matrix = Param::array()
    ///     .description("2D matrix of numbers")
    ///     .items(Param::array()
    ///         .items(Param::number().end())
    ///         .end())
    ///     .end();
    /// ```
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

    /// Creates a builder for a number parameter.
    ///
    /// Number parameters represent floating-point numeric values.
    /// Use this for prices, temperatures, percentages, and other
    /// values that may have decimal places.
    ///
    /// # Parameters
    ///
    /// * `description` - Optional description of the number's purpose
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orpheus::prelude::*;
    ///
    /// // Price parameter
    /// let price = Param::number()
    ///     .description("Price in USD")
    ///     .end();
    ///
    /// // Temperature parameter
    /// let temperature = Param::number()
    ///     .description("Temperature in Celsius")
    ///     .end();
    ///
    /// // Percentage parameter
    /// let confidence = Param::number()
    ///     .description("Confidence score between 0.0 and 1.0")
    ///     .end();
    /// ```
    #[builder(finish_fn = end)]
    pub fn number(#[builder(into)] description: Option<String>) -> Self {
        Self::Number { description }
    }

    /// Creates a builder for a boolean parameter.
    ///
    /// Boolean parameters represent true/false values. Use this for
    /// flags, switches, and binary choices.
    ///
    /// # Parameters
    ///
    /// * `description` - Optional description of the boolean's purpose
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orpheus::prelude::*;
    ///
    /// // Active flag
    /// let is_active = Param::boolean()
    ///     .description("Whether the user is currently active")
    ///     .end();
    ///
    /// // Feature flag
    /// let enable_notifications = Param::boolean()
    ///     .description("Enable push notifications")
    ///     .end();
    ///
    /// // Verification status
    /// let is_verified = Param::boolean()
    ///     .description("Whether the email is verified")
    ///     .end();
    /// ```
    #[builder(finish_fn = end)]
    pub fn boolean(#[builder(into)] description: Option<String>) -> Self {
        Self::Boolean { description }
    }
}

impl<S: param_object_builder::State> ParamObjectBuilder<S> {
    /// Adds a single property to the object parameter.
    ///
    /// This method allows you to define individual properties of an object
    /// parameter. Each property consists of a name (key) and its parameter
    /// definition (value). Properties can be of any valid parameter type.
    ///
    /// # Parameters
    ///
    /// * `key` - The property name (converted to String)
    /// * `value` - The parameter definition for this property
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orpheus::prelude::*;
    ///
    /// let user_schema = Param::object()
    ///     .property("id", Param::integer().description("User ID"))
    ///     .property("name", Param::string().description("Full name"))
    ///     .property("email", Param::string().description("Email address"))
    ///     .property("settings", Param::object()
    ///         .property("theme", Param::string().enums(["light", "dark"]))
    ///         .property("notifications", Param::boolean())
    ///         .end())
    ///     .required(["id", "name", "email"])
    ///     .end();
    /// ```
    pub fn property(mut self, key: impl Into<String>, value: impl Into<ParamType>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// Sets all properties at once using a HashMap.
    ///
    /// This method allows you to define multiple properties simultaneously
    /// by providing a HashMap of property names to parameter definitions.
    /// This can be useful when building schemas programmatically.
    ///
    /// # Parameters
    ///
    /// * `properties` - HashMap mapping property names to their parameter definitions
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orpheus::prelude::*;
    /// use std::collections::HashMap;
    ///
    /// let mut props = HashMap::new();
    /// props.insert("name".to_string(), Param::string().end().into());
    /// props.insert("age".to_string(), Param::integer().end().into());
    ///
    /// let schema = Param::object()
    ///     .properties(props)
    ///     .required(["name"])
    ///     .end();
    /// ```
    pub fn properties(mut self, properties: HashMap<String, ParamType>) -> Self {
        self.properties = properties;
        self
    }
}

/// A collection of tools that can be provided to a language model.
///
/// This wrapper struct holds a vector of tools and provides conversion
/// methods for easy integration with the chat API. Tools define functions
/// that the model can call during conversation.
#[derive(Debug, Clone, Serialize)]
pub struct Tools(pub Vec<Tool>);

impl From<Vec<Tool>> for Tools {
    /// Converts a vector of tools into a Tools collection.
    ///
    /// This provides a convenient way to create Tools from a `Vec<Tool>`.
    fn from(tools: Vec<Tool>) -> Self {
        Self(tools)
    }
}

impl<const N: usize> From<[Tool; N]> for Tools {
    /// Converts an array of tools into a Tools collection.
    ///
    /// This provides a convenient way to create Tools from a fixed-size array.
    fn from(tools: [Tool; N]) -> Self {
        Self(tools.to_vec())
    }
}

/// Trait for types that can be converted into a Param.
///
/// This trait allows various parameter builders to be seamlessly converted
/// into the final Param type. It's automatically implemented for all
/// parameter builders when they're in a complete state.
///
/// # Examples
///
/// ```rust
/// use orpheus::prelude::*;
///
/// fn create_string_param() -> impl Parameter {
///     Param::string().description("A string parameter")
/// }
///
/// let param: Param = create_string_param().into_param();
/// ```
pub trait Parameter {
    /// Converts this type into a Param.
    fn into_param(self) -> Param;
}

impl<T: Parameter> From<T> for ParamType {
    /// Converts any Parameter type into a ParamType.
    ///
    /// This allows parameter builders to be used anywhere a ParamType
    /// is expected, providing a seamless conversion pathway.
    fn from(value: T) -> Self {
        value.into_param().into()
    }
}

/// Macro to implement the Parameter trait for builder types.
///
/// This macro generates Parameter implementations for various parameter
/// builders, allowing them to be converted into Param instances when
/// they're in a complete state.
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

// Implement Parameter trait for all parameter builders
impl_parameter_for_builder!(ParamStringBuilder, param_string_builder);
impl_parameter_for_builder!(ParamObjectBuilder, param_object_builder);
impl_parameter_for_builder!(ParamArrayBuilder, param_array_builder);
impl_parameter_for_builder!(ParamNumberBuilder, param_number_builder);
impl_parameter_for_builder!(ParamBooleanBuilder, param_boolean_builder);
impl_parameter_for_builder!(ParamIntegerBuilder, param_integer_builder);

/// Creates a union type (anyOf) parameter from multiple parameter types.
///
/// This macro allows you to create parameters that can accept multiple
/// different types. It's useful for optional fields that could be different
/// types, or for flexible APIs that accept various input formats.
///
/// # Examples
///
/// ```rust
/// use orpheus::{anyof, models::chat::Param};
///
/// // Create a parameter that can be string, number, or null
/// let flexible_value = anyof![
///     Param::string(),
///     Param::number(),
///     Param::null()
/// ];
///
/// // Use in an object schema
/// let schema = Param::object()
///     .property("id", anyof![Param::string(), Param::integer()])
///     .property("optional_field", anyof![Param::string(), Param::null()])
///     .end();
/// ```
#[macro_export]
macro_rules! anyof {
    ($($param:expr),* $(,)?) => {{
        use $crate::models::chat::{ParamType, Parameter};

        let any_of: Vec<Param> = vec![$($param.into_param()),*];
        ParamType::Any { any_of }
    }};
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    /// Test that demonstrates building complex nested object parameters.
    ///
    /// This test shows how to create a weather tool parameter with nested
    /// location coordinates and enum constraints for units.
    #[test]
    fn test_shorthand_tool_dev() {
        // Expected JSON structure for a complex nested parameter
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

        // Create the same structure using the builder API
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

    /// Test that demonstrates creating arrays with union types (anyOf).
    ///
    /// This test shows how to create an array parameter where items can
    /// have optional fields using union types.
    #[test]
    fn deser_vec_tool_def() {
        // Expected JSON for array with union type properties
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

        // Create array with union type using anyof! macro
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

    /// Test that demonstrates creating tools with empty and simple parameters.
    ///
    /// This test shows the difference between tools with no parameters
    /// and tools with simple parameter objects.
    #[test]
    fn test_charades_tools() {
        // Test empty parameters tool
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

        // Test simple parameters tool
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
