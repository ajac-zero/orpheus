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

impl From<Plugin> for Vec<Plugin> {
    fn from(value: Plugin) -> Self {
        vec![value]
    }
}

#[bon]
impl Plugin {
    #[builder(finish_fn = build, on(String, into), on(i32, into))]
    fn web(max_results: Option<i32>, search_prompt: Option<String>) -> Self {
        Self::Web {
            max_results,
            search_prompt,
        }
    }

    #[builder(finish_fn = build)]
    fn file_parser(#[builder(field)] pdf: Option<ParsingEngine>) -> Self {
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
    use crate::{
        client::Orpheus,
        models::{ParsingEngine, Plugin},
    };

    #[test]
    fn create_web_plugin_with_builder() {
        let plugin = Plugin::web().build();

        assert_eq!(
            plugin,
            Plugin::Web {
                max_results: None,
                search_prompt: None
            }
        );

        let plugin = Plugin::web().max_results(10).build();

        assert_eq!(
            plugin,
            Plugin::Web {
                max_results: Some(10),
                search_prompt: None
            }
        );

        let plugin = Plugin::web().search_prompt("Relevant web results:").build();

        assert_eq!(
            plugin,
            Plugin::Web {
                max_results: None,
                search_prompt: Some("Relevant web results:".to_string())
            }
        );

        let plugin = Plugin::web()
            .max_results(10)
            .search_prompt("Relevant web results:")
            .build();

        assert_eq!(
            plugin,
            Plugin::Web {
                max_results: Some(10),
                search_prompt: Some("Relevant web results:".to_string())
            }
        );
    }

    #[test]
    fn create_parser_plugin_with_builder() {
        let plugin = Plugin::file_parser().build();

        assert_eq!(
            plugin,
            Plugin::FileParser {
                pdf: ParsingEngine::default()
            }
        );

        let plugin = Plugin::file_parser()
            .engine(ParsingEngine::MistralOcr)
            .build();

        assert_eq!(
            plugin,
            Plugin::FileParser {
                pdf: ParsingEngine::MistralOcr
            }
        );

        let plugin = Plugin::file_parser()
            .try_engine("mistral-ocr")
            .unwrap()
            .build();

        assert_eq!(
            plugin,
            Plugin::FileParser {
                pdf: ParsingEngine::MistralOcr
            }
        );
    }

    #[test]
    fn request_with_web_plugin() {
        let client = Orpheus::from_env().unwrap();

        let _ = client
            .chat("What is the capital of France?")
            .model("openai/gpt-4o")
            .plugins(Plugin::web().build());
    }

    #[test]
    fn request_with_file_plugin() {
        let client = Orpheus::from_env().unwrap();

        let _ = client
            .chat("What is the capital of France?")
            .model("openai/gpt-4o")
            .plugins(Plugin::file_parser().build());
    }

    #[test]
    fn request_with_multiple_plugins() {
        let client = Orpheus::from_env().unwrap();

        let plugins = vec![
            Plugin::web().max_results(10).build(),
            Plugin::file_parser().build(),
        ];

        let _ = client
            .chat("What is the capital of France?")
            .model("openai/gpt-4o")
            .plugins(plugins);
    }
}
