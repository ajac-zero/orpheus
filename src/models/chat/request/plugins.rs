use bon::bon;
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(tag = "engine", rename_all = "kebab-case")]
pub enum ParsingEngine {
    PdfText,
    #[default]
    MistralOcr,
    Native,
}

impl TryFrom<String> for ParsingEngine {
    type Error = Error;

    fn try_from(value: String) -> Result<ParsingEngine> {
        match value.as_str() {
            "pdf-text" => Ok(ParsingEngine::PdfText),
            "mistral-ocr" => Ok(ParsingEngine::MistralOcr),
            "native" => Ok(ParsingEngine::Native),
            _ => Err(Error::invalid_parsing_engine(value)),
        }
    }
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "id", rename_all = "kebab-case")]
pub enum Plugin {
    FileParser {
        pdf: ParsingEngine,
    },
    Web {
        max_results: Option<i32>,
        search_prompt: Option<String>,
    },
}

#[bon]
impl Plugin {
    #[builder(finish_fn = build, on(String, into), on(i32, into))]
    pub fn web(max_results: Option<i32>, search_prompt: Option<String>) -> Self {
        Self::Web {
            max_results,
            search_prompt,
        }
    }

    #[builder(finish_fn = build)]
    pub fn file_parser(#[builder(field)] pdf: Option<ParsingEngine>) -> Self {
        Self::FileParser {
            pdf: pdf.unwrap_or_else(ParsingEngine::default),
        }
    }
}

impl<S: plugin_file_parser_builder::State> PluginFileParserBuilder<S> {
    pub fn engine(mut self, engine: impl Into<ParsingEngine>) -> PluginFileParserBuilder<S> {
        self.pdf = Some(engine.into());
        self
    }

    pub fn try_engine(mut self, engine: impl Into<String>) -> Result<PluginFileParserBuilder<S>> {
        self.pdf = Some(engine.into().try_into()?);
        Ok(self)
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::models::{ParsingEngine, Plugin};

    #[test]
    fn serialize_file_parser_plugin() {
        let target = json!({
              "id": "file-parser",
              "pdf": {
                "engine": "pdf-text", // or 'mistral-ocr' or 'native'
              },
        });

        let value = Plugin::file_parser().engine(ParsingEngine::PdfText).build();
        let result = serde_json::to_value(value).unwrap();

        assert_eq!(target, result);
    }

    #[test]
    fn serialize_web_plugin() {
        let target = json!({"id": "web"});

        let value = Plugin::web().build();
        let result = serde_json::to_value(value).unwrap();

        assert_eq!(target, result);
    }

    #[test]
    fn serialize_web_plugin_with_options() {
        let target = json!({
            "id": "web",
            "max_results": 10,
            "search_prompt": "Some relevant web results:"
        });

        let value = Plugin::web()
            .max_results(10)
            .search_prompt("Some relevant web results:")
            .build();
        let result = serde_json::to_value(value).unwrap();

        assert_eq!(target, result);
    }
}
