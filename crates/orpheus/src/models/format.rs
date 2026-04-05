use bon::bon;
use serde::{Deserialize, Serialize};

use crate::models::tool::{Param, ParamObjectBuilder, param_object_builder};

/// Represents the format specification for structured output responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "json_schema", rename_all = "snake_case")]
pub enum Format {
    /// JSON Schema format for structured output.
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

impl From<Format> for open_responses::TextParam {
    fn from(format: Format) -> Self {
        match format {
            Format::JsonSchema {
                name,
                strict,
                schema,
            } => {
                let schema_value = serde_json::to_value(schema).unwrap_or_default();
                open_responses::TextParam {
                    format: Some(open_responses::TextFormatParam::JsonSchema(
                        open_responses::JsonSchemaResponseFormatParam {
                            type_: "json_schema".into(),
                            name,
                            description: None,
                            schema: schema_value,
                            strict: Some(strict),
                        },
                    )),
                    verbosity: None,
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::models::{Format, Param};

    #[test]
    fn serialize_response_format() {
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

        let response_format_value = serde_json::to_value(response_format).unwrap();

        assert_eq!(target, response_format_value)
    }
}
