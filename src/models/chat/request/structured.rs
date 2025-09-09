use bon::bon;
use serde::{Deserialize, Serialize};

use crate::models::chat::{
    Param,
    request::tool::{ParamObjectBuilder, param_object_builder},
};

/// Represents the format specification for structured output responses.
///
/// This enum defines how the model should format its response, particularly for
/// generating structured JSON data that conforms to a specific schema.
///
/// # Example
/// ```rust
/// use orpheus::prelude::*;
///
/// let client = Client::from_env().unwrap();
///
/// // Create a simple person schema
/// let format = Format::json("person")
///     .with_schema(|schema| {
///         schema
///             .property("name", Param::string())
///             .property("age", Param::number())
///             .property("city", Param::string())
///             .required(["name", "age"])
///     })
///     .build();
///
/// // Use schema to generate a structured response
/// let response = client
///     .chat("Hello, how old are you?")
///     .model("openai/gpt-4o")
///     .response_format(format)
///     .await
///     .unwrap();
///
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "json_schema", rename_all = "snake_case")]
pub enum Format {
    /// JSON Schema format for structured output.
    ///
    /// This variant specifies that the model should return JSON data that
    /// conforms to the provided schema definition.
    JsonSchema {
        /// The name identifier for this schema
        name: String,
        /// Whether to enforce strict adherence to the schema
        strict: bool,
        /// The parameter definition that describes the expected JSON structure
        schema: Param,
    },
}

#[bon]
impl Format {
    /// Creates a new JSON schema format builder.
    ///
    /// This is the primary entry point for defining structured output formats.
    /// The builder pattern allows you to specify the schema name, strict mode,
    /// and the actual schema definition.
    ///
    /// # Parameters
    ///
    /// * `name` - A unique identifier for this schema
    /// * `strict` - Whether to enforce strict schema validation (default: true)
    /// * `with_schema/schema` - The parameter definition describing the expected structure; `with_schema` starts a builder; `schema` expects the complete `Param` object.
    ///
    /// # Examples
    /// ```rust
    /// use orpheus::prelude::*;
    ///
    /// // Basic usage with default strict mode and `with_schema`
    /// let format = Format::json("user_data")
    ///     .with_schema(|schema| {
    ///         schema
    ///             .property("name", Param::string())
    ///             .property("age", Param::integer())
    ///             .required(["name"])
    ///     })
    ///     .build();
    ///
    /// // With explicit strict mode disabled and `schema`
    /// let format = Format::json("flexible_data")
    ///     .strict(false)
    ///     .schema(
    ///         Param::object()
    ///             .property("name", Param::string())
    ///             .property("age", Param::integer())
    ///             .required(["name"])
    ///             .end()
    ///     )
    ///     .build();
    /// ```
    #[builder(finish_fn = build)]
    pub fn json(
        #[builder(into, start_fn)] name: String,
        #[builder(default = true)] strict: bool,
        schema: Param,
    ) -> Self {
        Self::JsonSchema {
            name,
            strict,
            schema,
        }
    }
}

impl<S: format_json_builder::State> FormatJsonBuilder<S> {
    /// Defines the schema structure using a builder closure.
    ///
    /// This method provides an ergonomic way to define complex object schemas
    /// without having to manually create and configure `Param::object()` builders.
    /// The closure receives a `ParamObjectBuilder` that you can use to define
    /// properties, requirements, and other schema constraints.
    ///
    /// # Parameters
    ///
    /// * `build` - A closure that receives a `ParamObjectBuilder` and returns
    ///   a configured builder with the desired schema structure
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orpheus::prelude::*;
    ///
    /// let format = Format::json("weather")
    ///     .with_schema(|schema| {
    ///         schema
    ///             .property("location", Param::string().description("City name"))
    ///             .property("temperature", Param::number().description("Temperature in Celsius"))
    ///             .property("conditions", Param::string().description("Weather description"))
    ///             .required(["location", "temperature", "conditions"])
    ///     })
    ///     .build();
    /// ```
    ///
    /// # Note
    ///
    /// This method automatically sets `additional_properties(false)` to ensure
    /// the generated JSON strictly adheres to the defined schema.
    pub fn with_schema<F, C>(self, build: F) -> FormatJsonBuilder<format_json_builder::SetSchema<S>>
    where
        S::Schema: format_json_builder::IsUnset,
        F: FnOnce(ParamObjectBuilder<param_object_builder::Empty>) -> ParamObjectBuilder<C>,
        C: param_object_builder::IsComplete,
        C::AdditionalProperties: param_object_builder::IsUnset,
    {
        let builder = Param::object();
        let param = build(builder).additional_properties(false).end();
        self.schema(param)
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::prelude::{Format, Orpheus, Param};

    /// Tests that a Format with a complex schema serializes to the expected JSON structure.
    ///
    /// This test verifies that the builder API correctly generates the JSON schema
    /// format expected by OpenRouter and compatible APIs.
    #[test]
    fn serialize_response_format() {
        // Expected JSON structure that should be generated by the Format builder
        let target = json!({
          "type": "json_schema",
          "json_schema": {
            "name": "weather",
            "strict": true,
            "schema": {
              "type": "object",
              "properties": {
                "location": {
                  "type": "string",
                  "description": "City or location name"
                },
                "temperature": {
                  "type": "number",
                  "description": "Temperature in Celsius"
                },
                "conditions": {
                  "type": "string",
                  "description": "Weather conditions description"
                }
              },
              "required": ["location", "temperature", "conditions"],
              "additionalProperties": false
            }
          }
        });

        // Create a Format using the builder API with the same structure as the target
        let response_format = Format::json("weather")
            .with_schema(|schema| {
                schema
                    .property(
                        "location",
                        Param::string().description("City or location name"),
                    )
                    .property(
                        "temperature",
                        Param::number().description("Temperature in Celsius"),
                    )
                    .property(
                        "conditions",
                        Param::string().description("Weather conditions description"),
                    )
                    .required(["location", "temperature", "conditions"])
            })
            .build();

        // Serialize the Format and compare with expected structure
        let response_format_value = serde_json::to_value(response_format).unwrap();

        assert_eq!(target, response_format_value)
    }

    /// Integration test that demonstrates structured output in a real API call.
    ///
    /// This test shows how to use structured output end-to-end, from defining
    /// the schema to making the API call and deserializing the response.
    ///
    /// Note: This test requires valid API credentials and may make actual API calls.
    #[test]
    fn end_to_end_with_response_format() {
        // Initialize client from environment variables (ORPHEUS_API_KEY)
        let client = Orpheus::from_env().unwrap();

        // Define the expected response format for weather data
        let response_format = Format::json("weather")
            .with_schema(|schema| {
                schema
                    .property(
                        "location",
                        Param::string().description("City or location name"),
                    )
                    .property(
                        "temperature",
                        Param::number().description("Temperature in Celsius"),
                    )
                    .property(
                        "conditions",
                        Param::string().description("Weather conditions description"),
                    )
                    .required(["location", "temperature", "conditions"])
            })
            .build();

        // Make a chat request with structured output format
        let response = client
            .chat("What is the weather like in New York City?")
            .model("openai/gpt-4o")
            .response_format(response_format)
            .send()
            .unwrap();

        /// Struct that matches our defined schema for easy deserialization
        #[derive(Debug, serde::Deserialize)]
        struct WeatherResponse {
            location: String,
            temperature: f64,
            conditions: String,
        }

        // Extract the JSON content from the response
        let content = response.content().unwrap().to_string();

        // Deserialize the structured JSON response into our struct
        let weather_response: WeatherResponse = serde_json::from_str(&content).unwrap();

        // Print the parsed response for debugging
        dbg!(weather_response);
    }
}
