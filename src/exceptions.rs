use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrpheusError {
    #[error("OpenRouter error: {0}")]
    OpenRouter(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("String {0} did not parse as valid URL")]
    InvalidUrl(#[from] url::ParseError),
    #[error("Error making the request: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Invalid SSE line: {0}")]
    InvalidSSE(String),
}
