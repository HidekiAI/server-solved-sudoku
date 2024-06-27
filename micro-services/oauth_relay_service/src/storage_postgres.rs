use tokio_postgres::{NoTls, Error};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const CREATE_TABLE: &str = r#"  
CREATE TABLE IF NOT EXISTS tokens (
    client_id TEXT PRIMARY KEY,
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    expires_in INTEGER NOT NULL,
    expiry_time INTEGER NOT NULL
)"#;

#[derive(Clone, Debug)]
pub struct TokenData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub expiry_time: SystemTime,
}

async fn store_token(client: &tokio_postgres::Client, client_id: &str, token_data: &TokenData) -> Result<(), Error> {
    client.execute(
        "INSERT INTO tokens (client_id, access_token, refresh_token, expires_in, expiry_time) VALUES ($1, $2, $3, $4, $5)",
        &[&client_id, &token_data.access_token, &token_data.refresh_token, &token_data.expires_in, &token_data.expiry_time.duration_since(UNIX_EPOCH)?.as_secs() as i64],
    ).await?;
    Ok(())
}

async fn get_token(client: &tokio_postgres::Client, client_id: &str) -> Result<Option<TokenData>, Error> {
    let row = client.query_opt(
        "SELECT access_token, refresh_token, expires_in, expiry_time FROM tokens WHERE client_id = $1",
        &[&client_id],
    ).await?;

    if let Some(row) = row {
        let token_data = TokenData {
            access_token: row.get(0),
            refresh_token: row.get(1),
            expires_in: row.get(2),
            expiry_time: UNIX_EPOCH + Duration::from_secs(row.get::<_, i64>(3) as u64),
        };
        Ok(Some(token_data))
    } else {
        Ok(None)
    }
}
