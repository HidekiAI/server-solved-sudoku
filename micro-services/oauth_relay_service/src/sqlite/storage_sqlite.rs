use super::TConnection;
use crate::data::{SessionIDType, TokenData};
use anyhow::Result as AnyResult;
use std::time::UNIX_EPOCH;
use tokio_rusqlite::{self, params, Connection};

// Note that we use "IF NOT EXISTS" so that create_table_token() can be
// safely called multiple times without failing...
const CREATE_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS tokens (
    session_id    INTEGER PRIMARY KEY AUTOINCREMENT,

    client_address TEXT NOT NULL,
    client_port   INTEGER NOT NULL,
    client_email  TEXT,

    access_token  TEXT NOT NULL,
    refresh_token TEXT,
    expires_in    INTEGER NOT NULL,
    expiry_time   INTEGER NOT NULL)
"#;
pub(crate) async fn create_table_token(db_connection: TConnection) -> tokio_rusqlite::Result<()> {
    let conn = db_connection.lock().await;
    conn.call(|conn| match conn.execute(CREATE_TABLE, []) {
        Ok(_) => Ok(()),
        Err(e) => Err(tokio_rusqlite::Error::Rusqlite(e)),
    })
    .await
}

// Note that session_id is primary key and should be returned to caller via conn.last_insert_rowid()
// It's weird that sqlite::rowdata is 0'based (i.e. col1=row.get(0)?;), yet statements are 1'based...
// IN1: client_address (TEXT)
// IN2: client_port (INTEGER)
// IN3: client_email (optional TEXT)
// IN4: access_token (TEXT (from server))
// IN5: refresh_token (optional TEXT)
// IN6: expires_in: (INTEGER (from server))
// IN7: expiry_time (INTEGER (epoch time))
const INSERT_TOKEN: &str = r#"
INSERT INTO tokens (
        client_address, client_port, client_email, 
        access_token, refresh_token, expires_in, expiry_time) 
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
"#;
pub(crate) async fn store_token(
    db_connection: TConnection,
    token_data_no_session_id: TokenData,
) -> tokio_rusqlite::Result<i64> {
    let conn = db_connection.lock().await;
    conn.call(move |conn| {
        conn.execute(
            INSERT_TOKEN,
            params![
                token_data_no_session_id.client_address.to_string(), // 1
                token_data_no_session_id.client_port,                // 2
                token_data_no_session_id.possible_client_email,      // 3
                token_data_no_session_id.access_token,               // 4
                token_data_no_session_id.refresh_token,              // 5
                token_data_no_session_id.expires_in,                 // 6
                (token_data_no_session_id
                    .expiry_time
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64), // 7
            ],
        )?;
        let session_id = conn.last_insert_rowid();
        Ok(session_id)
    })
    .await
}

// IN1: session_id (INTEGER)
const SELECT_TOKEN: &str = r#"
SELECT client_address, 
        client_port, client_email, 
        access_token, refresh_token, expires_in, expiry_time 
    FROM tokens WHERE session_id = ?1
"#;
pub(crate) async fn get_token_by_session_id(
    db_connection: TConnection,
    session_id: u64,
) -> tokio_rusqlite::Result<TokenData> {
    let conn = db_connection.lock().await;
    conn.call(move |conn| {
        let mut stmt = conn.prepare(SELECT_TOKEN)?;
        // in parm1: session_id
        let token_data = stmt.query_row(params![session_id], |row| {
            Ok(TokenData::new(
                SessionIDType::ID(session_id),
                row.get::<usize, String>(0)?.to_string().parse().unwrap(),
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                (UNIX_EPOCH + std::time::Duration::new(row.get(6)?, 10)),
            ))
        })?;
        Ok(token_data)
    })
    .await
}
