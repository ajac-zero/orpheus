use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "engine", rename_all = "kebab-case")]
pub enum ParsingEngine {
    PdfText,
    MistralOcr,
    Native,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "id", rename_all = "kebab-case")]
pub enum Plugin {
    FileParser {
        pdf: ParsingEngine,
    },
    Web {
        max_results: Option<u64>,
        search_prompt: Option<String>,
    },
}
