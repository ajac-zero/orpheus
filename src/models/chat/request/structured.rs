use bon::bon;
use serde::{Deserialize, Serialize};

use crate::{
    Param,
    models::chat::request::tool::{ParamObjectBuilder, param_object_builder},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "json_schema", rename_all = "snake_case")]
pub enum ResponseFormat {
    JsonSchema {
        name: String,
        strict: bool,
        schema: Param,
    },
}

#[bon]
impl ResponseFormat {
    #[builder(finish_fn = build)]
    pub fn json_schema(
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

impl<S: response_format_json_schema_builder::State> ResponseFormatJsonSchemaBuilder<S> {
    pub fn with_schema<F, C>(
        self,
        build: F,
    ) -> ResponseFormatJsonSchemaBuilder<response_format_json_schema_builder::SetSchema<S>>
    where
        S::Schema: response_format_json_schema_builder::IsUnset,
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

    use crate::{Orpheus, Param, models::chat::request::structured::ResponseFormat};

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

        let response_format = ResponseFormat::json_schema("weather")
            .with_schema(|schema| {
                schema
                    .property(
                        "location",
                        Param::string().description("City or location name").end(),
                    )
                    .property(
                        "temperature",
                        Param::number().description("Temperature in Celsius").end(),
                    )
                    .property(
                        "conditions",
                        Param::string()
                            .description("Weather conditions description")
                            .end(),
                    )
                    .required(["location", "temperature", "conditions"])
            })
            .build();

        let response_format_value = serde_json::to_value(response_format).unwrap();

        assert_eq!(target, response_format_value)
    }

    #[test]
    fn end_to_end_with_response_format() {
        let client = Orpheus::from_env().unwrap();

        let response_format = ResponseFormat::json_schema("weather")
            .with_schema(|schema| {
                schema
                    .property(
                        "location",
                        Param::string().description("City or location name").end(),
                    )
                    .property(
                        "temperature",
                        Param::number().description("Temperature in Celsius").end(),
                    )
                    .property(
                        "conditions",
                        Param::string()
                            .description("Weather conditions description")
                            .end(),
                    )
                    .required(["location", "temperature", "conditions"])
            })
            .build();

        let response = client
            .chat("What is the weather like in New York City?")
            .model("openai/gpt-4o")
            .response_format(response_format)
            .send()
            .unwrap();

        #[derive(Debug, serde::Deserialize)]
        struct WeatherResponse {
            location: String,
            temperature: f64,
            conditions: String,
        }

        let content = response.content().unwrap().to_string();

        let weather_response: WeatherResponse = serde_json::from_str(&content).unwrap();
        dbg!(weather_response);
    }
}
