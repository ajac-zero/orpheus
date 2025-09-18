use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct OpenRouterErrorResponse {
    pub error: OpenRouterError,
}

#[derive(Debug, Deserialize)]
pub struct OpenRouterError {
    pub message: String,
    pub code: u16,
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

    #[error("Missing {0} key")]
    MissingKey(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("de/serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("http error: {0}")]
    Http(#[from] hyper::http::Error),

    #[error("OpenRouter Error {code}: {message}")]
    OpenRouter { code: u16, message: String },

    #[error("HTTP {status}: {body}")]
    Unexpected { status: u16, body: String },

    #[error("Tokio task error: {0}")]
    Tokio(#[from] tokio::task::JoinError),

    #[cfg(feature = "mcp")]
    #[error("MCP error: {0}")]
    Mcp(#[from] rmcp::ServiceError),

    #[cfg(feature = "mcp")]
    #[error("MCP init error: {0}")]
    McpInit(#[from] rmcp::service::ClientInitializeError<std::io::Error>),
}

impl OrpheusError {
    pub(crate) fn invalid_parsing_engine(engine: String) -> Self {
        OrpheusError::Parsing(format!("Invalid parsing engine: {}", engine))
    }

    pub(crate) fn invalid_sse(line: &str) -> Self {
        OrpheusError::Parsing(format!("Invalid sse line: {}", line))
    }

    pub(crate) fn invalid_role(role: &str) -> Self {
        OrpheusError::Parsing(format!("Invalid role: {}", role))
    }

    pub(crate) fn missing_choices_array() -> Self {
        OrpheusError::Parsing("Choices array in response is empty".into())
    }

    pub(crate) fn missing_api_key() -> Self {
        Self::MissingKey("api".into())
    }

    pub(crate) fn missing_provisioning_key() -> Self {
        Self::MissingKey("provisioning".into())
    }

    pub(crate) fn openrouter_error(code: u16, message: String) -> Self {
        Self::OpenRouter { code, message }
    }

    pub(crate) fn unexpected_http_status(status: u16, body: String) -> Self {
        Self::Unexpected { status, body }
    }

    pub(crate) fn parse_openrouter_error(status: u16, body: &str) -> Self {
        match serde_json::from_str::<OpenRouterErrorResponse>(body) {
            Ok(api_error_response) => Self::openrouter_error(
                api_error_response.error.code,
                api_error_response.error.message,
            ),
            Err(_) => {
                // If we can't parse as structured error, fall back to raw HTTP error
                Self::unexpected_http_status(status, body.to_string())
            }
        }
    }

    #[cfg(feature = "mcp")]
    pub(crate) fn missing_tool_schema_key(missing_key: &str) -> Self {
        OrpheusError::Parsing(format!("Invalid tool schema; Missing key: {}", missing_key))
    }
}
