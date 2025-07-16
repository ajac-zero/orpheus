use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPreferences {
    /// Sort preference (e.g., price, throughput).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
}
