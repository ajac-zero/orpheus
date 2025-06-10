#[derive(Debug)]
pub enum OrpheusError {
    Http(reqwest::Error),
    Serialization(serde_json::Error),
    ApiError { status: u16, message: String },
    MissingApiKey,
    Anyhow(String),
    Io(tokio::io::Error),
}

impl From<anyhow::Error> for OrpheusError {
    fn from(err: anyhow::Error) -> Self {
        OrpheusError::Anyhow(err.to_string())
    }
}

impl From<reqwest::Error> for OrpheusError {
    fn from(err: reqwest::Error) -> Self {
        OrpheusError::Http(err)
    }
}

impl From<serde_json::Error> for OrpheusError {
    fn from(err: serde_json::Error) -> Self {
        OrpheusError::Serialization(err)
    }
}

impl From<tokio::io::Error> for OrpheusError {
    fn from(err: tokio::io::Error) -> Self {
        OrpheusError::Io(err)
    }
}

impl std::fmt::Display for OrpheusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrpheusError::Http(e) => write!(f, "HTTP error: {}", e),
            OrpheusError::Serialization(e) => write!(f, "Serialization error: {}", e),
            OrpheusError::ApiError { status, message } => {
                write!(f, "API error {}: {}", status, message)
            }
            OrpheusError::MissingApiKey => write!(f, "API key is required"),
            OrpheusError::Anyhow(e) => write!(f, "Anyhow error: {}", e),
            OrpheusError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for OrpheusError {}
