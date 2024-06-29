use super::data_sqlite::{AuthRequest, SharedTokens, TokenData, TokenResponse};
use super::data_sqlite::TokenData;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use std::time::{SystemTime, UNIX_EPOCH};

const CREATE_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS tokens (
    client_id TEXT PRIMARY KEY,
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    expires_in INTEGER NOT NULL,
    expiry_time INTEGER NOT NULL
)"#;

pub(crate) fn store_token(
    conn: &Connection,
    client_id: &str,
    token_data: &TokenData,
) -> Result<()> {
    conn.execute(
        "INSERT INTO tokens (client_id, access_token, refresh_token, expires_in, expiry_time) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            client_id,
            token_data.access_token,
            token_data.refresh_token,
            token_data.expires_in,
            token_data.expiry_time.duration_since(UNIX_EPOCH)?.as_secs() ,
        ],
    )?;
    Ok(())
}

fn get_token(conn: &Connection, client_id: &str) -> Result<Option<TokenData>> {
    let mut stmt = conn.prepare("SELECT access_token, refresh_token, expires_in, expiry_time FROM tokens WHERE client_id = ?1")?;
    let token_iter = stmt.query_map(params![client_id], |row| {
        Ok(TokenData {
            access_token: row.get(0)?,
            refresh_token: row.get(1)?,
            expires_in: row.get(2)?,
            expiry_time: UNIX_EPOCH + std::time::Duration::from_secs(row.get::<_, i64>(3)? as u64),
        })
    })?;

    for token in token_iter {
        return Ok(Some(token?));
    }
    Ok(None)
}
