use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
struct ApiErrorEnvelope {
    error: ApiErrorBody,
}

#[derive(Debug, Deserialize)]
struct ApiErrorBody {
    message: String,
    #[serde(default)]
    code: Option<String>,
}

#[derive(Error, Debug)]
pub enum OrpheusError {
    #[error("Missing env var: {0}")]
    Env(#[from] std::env::VarError),

    #[error("String {0} did not parse as valid URL")]
    InvalidUrl(#[from] url::ParseError),

    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),

    #[error("Parsing error: {0}")]
    Parsing(String),

    #[error("Missing API key")]
    MissingKey,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("de/serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("http error: {0}")]
    Http(#[from] hyper::http::Error),

    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },

    #[error("HTTP {status}: {body}")]
    Unexpected { status: u16, body: String },

    #[error("Tokio task error: {0}")]
    Tokio(#[from] tokio::task::JoinError),

    #[error("Unknown stream event: {0}")]
    UnknownStreamEvent(String),
}

impl OrpheusError {
    pub(crate) fn invalid_sse(line: &str) -> Self {
        OrpheusError::Parsing(format!("Invalid SSE line: {}", line))
    }

    pub(crate) fn missing_api_key() -> Self {
        Self::MissingKey
    }

    pub(crate) fn parse_api_error(status: u16, body: &str) -> Self {
        match serde_json::from_str::<ApiErrorEnvelope>(body) {
            Ok(envelope) => Self::Api {
                status,
                message: envelope.error.message,
            },
            Err(_) => Self::Unexpected {
                status,
                body: body.to_string(),
            },
        }
    }
}
