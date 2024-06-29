use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

pub type SharedTokens = Arc<Mutex<HashMap<String, TokenData>>>;

#[derive(Serialize, Deserialize)]
pub struct TokenData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub expiry_time: SystemTime,
}

#[derive(Deserialize)]
pub struct AuthRequest {
    pub code: String,
}

#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub scope: String,
    pub id_token: String,
}
