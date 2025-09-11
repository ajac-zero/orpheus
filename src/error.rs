use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrpheusError {
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),

    #[error("OpenRouter error: {0}")]
    OpenRouter(#[from] OpenRouterError),

    #[error("Request error: {0}")]
    Request(#[from] RequestError),

    #[error("Runtime error: {0}")]
    Runtime(#[from] RuntimeError),

    #[error("Parsing error: {0}")]
    Parsing(String),

    #[cfg(feature = "mcp")]
    #[error("MCP error: {0}")]
    Mcp(#[from] McpError),
}

impl OrpheusError {
    pub(crate) fn parse_error(error: impl Into<String>) -> Self {
        OrpheusError::Parsing(error.into())
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
pub enum RuntimeError {
    #[error("de/serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl OrpheusError {
    pub fn serde(error: serde_json::Error) -> Self {
        Self::Runtime(RuntimeError::Serde(error))
    }

    pub fn io(error: std::io::Error) -> Self {
        Self::Runtime(RuntimeError::Io(error))
    }
}

#[derive(Error, Debug)]
pub enum OpenRouterError {
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl OrpheusError {
    pub fn openrouter(error: impl Into<String>) -> Self {
        Self::OpenRouter(OpenRouterError::Unexpected(error.into()))
    }
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("Invalid SSE line: {0}")]
    InvalidSSE(String),

    #[error("Malformed response: {0}")]
    MalformedResponse(String),

    #[error("Error making the request: {0}")]
    Http(#[from] reqwest::Error),
}

impl OrpheusError {
    pub fn invalid_sse(error: impl Into<String>) -> Self {
        Self::Request(RequestError::InvalidSSE(error.into()))
    }

    pub fn malformed_response(error: impl Into<String>) -> Self {
        Self::Request(RequestError::MalformedResponse(error.into()))
    }

    pub fn http(error: reqwest::Error) -> Self {
        Self::Request(RequestError::Http(error))
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
