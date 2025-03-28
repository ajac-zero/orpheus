use pyo3::{FromPyObject, IntoPyObject};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, FromPyObject)]
pub enum EmbeddingInput {
    String(String),
    StringVector(Vec<String>),
    IntegerVector(Vec<f64>),
    NestedIntegerVector(Vec<Vec<f64>>),
}

#[derive(Debug, Serialize)]
pub struct EmbeddingPrompt {
    input: EmbeddingInput,
    model: String,
    encoding_format: Option<String>,
    dimensions: Option<i32>,
    user: Option<String>,
}

impl EmbeddingPrompt {
    pub fn new(
        input: EmbeddingInput,
        model: String,
        encoding_format: Option<String>,
        dimensions: Option<i32>,
        user: Option<String>,
    ) -> Self {
        Self {
            input,
            model,
            encoding_format,
            dimensions,
            user,
        }
    }
}

#[derive(Debug, Deserialize, IntoPyObject)]
pub struct EmbeddingResponse {
    index: i32,
    embedding: Vec<f64>,
    object: String,
}
