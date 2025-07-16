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
    #[error("Invalid response: {0}")]
    Response(String),
    #[error("MCP related error: {0}")]
    Mcp(String),
    #[error("{0}")]
    McpService(#[from] rmcp::ServiceError),
    #[error("{0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("Missing env var: {0}")]
    Env(#[from] std::env::VarError),
    #[cfg(feature = "anyhow")]
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}
