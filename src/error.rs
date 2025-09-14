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
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),

    #[error("Request error: {0}")]
    Request(#[from] RequestError),

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

    #[cfg(feature = "mcp")]
    #[error("MCP error: {0}")]
    Mcp(#[from] McpError),
}

impl OrpheusError {
    pub(crate) fn parse_error(error: impl Into<String>) -> Self {
        OrpheusError::Parsing(error.into())
    }

    pub(crate) fn missing_api_key() -> Self {
        Self::MissingKey("api".into())
    }

    pub(crate) fn missing_provisioning_key() -> Self {
        Self::MissingKey("provisioning".into())
    }

    pub fn openrouter_error(code: u16, message: String) -> Self {
        Self::OpenRouter { code, message }
    }

    pub fn unexpected_http_status(status: u16, body: String) -> Self {
        Self::Unexpected { status, body }
    }

    pub fn parse_openrouter_error(status: u16, body: &str) -> Self {
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
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing env var: {0}")]
    Env(#[from] std::env::VarError),

    #[error("String {0} did not parse as valid URL")]
    InvalidUrl(#[from] url::ParseError),

    #[error("Invalid parsing engine: {0}")]
    InvalidParsingEngine(String),
}

impl OrpheusError {
    pub fn env(error: std::env::VarError) -> Self {
        OrpheusError::Config(ConfigError::Env(error))
    }

    pub fn invalid_url(error: url::ParseError) -> Self {
        OrpheusError::Config(ConfigError::InvalidUrl(error))
    }

    pub fn invalid_parsing_engine(engine: String) -> Self {
        OrpheusError::Config(ConfigError::InvalidParsingEngine(engine))
    }
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("Invalid SSE line: {0}")]
    InvalidSSE(String),

    #[error("Malformed response: {0}")]
    MalformedResponse(String),
}

impl OrpheusError {
    pub fn invalid_sse(error: impl Into<String>) -> Self {
        Self::Request(RequestError::InvalidSSE(error.into()))
    }

    pub fn malformed_response(error: impl Into<String>) -> Self {
        Self::Request(RequestError::MalformedResponse(error.into()))
    }
}

#[cfg(feature = "mcp")]
#[derive(Error, Debug)]
pub enum McpError {
    #[error("MCP service error: {0}")]
    Service(#[from] rmcp::ServiceError),

    #[error("MCP initialization error: {0}")]
    Init(String),

    #[error("error closing the service: {0}")]
    Close(#[from] tokio::task::JoinError),

    #[error("{0}")]
    ToolSchema(String),
}

#[cfg(feature = "mcp")]
impl OrpheusError {
    pub fn service(error: rmcp::ServiceError) -> Self {
        Self::Mcp(McpError::Service(error))
    }

    pub fn init(error: impl Into<String>) -> Self {
        Self::Mcp(McpError::Init(error.into()))
    }

    pub fn close(error: tokio::task::JoinError) -> Self {
        Self::Mcp(McpError::Close(error))
    }

    pub fn tool_schema(error: impl Into<String>) -> Self {
        Self::Mcp(McpError::ToolSchema(error.into()))
    }
}
