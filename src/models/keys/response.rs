use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateKeyResult {
    pub data: ApiKey,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKey {
    pub hash: String,
    pub label: String,
    pub name: String,
    pub disabled: bool,
    pub limit: Option<i32>,
    pub usage: i32,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteKeyResult {
    pub deleted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListKeysResult {
    pub data: ApiKey,
}
