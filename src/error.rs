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

    #[cfg(feature = "mcp")]
    #[error("MCP error: {0}")]
    Mcp(#[from] McpError),

    #[cfg(feature = "anyhow")]
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing env var: {0}")]
    Env(#[from] std::env::VarError),

    #[error("String {0} did not parse as valid URL")]
    InvalidUrl(#[from] url::ParseError),
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("de/serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum OpenRouterError {
    #[error("Unexpected error: {0}")]
    Unexpected(String),
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
