use crate::data::{SessionIDType, TokenData};
//use anyhow::Result as AnyResult;
use std::time::UNIX_EPOCH;
use tokio_rusqlite::{self, params};

use super::{TDBConnectionLock_sqlite};

// Note that we use "IF NOT EXISTS" so that create_table_token() can be
// safely called multiple times without failing...
const CREATE_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS tokens (
    session_id      INTEGER PRIMARY KEY AUTOINCREMENT,
    state_token     TEXT NOT NULL UNIQUE,

    client_address  TEXT NOT NULL,
    client_port     INTEGER NOT NULL,
    client_email    TEXT,

    access_token    TEXT NOT NULL UNIQUE,
    refresh_token   TEXT,
    expires_in      INTEGER NOT NULL,
    expiry_time     INTEGER NOT NULL)
"#;
pub(crate) async fn create_table_token(
    db_connection: &TDBConnectionLock_sqlite,
) -> tokio_rusqlite::Result<()> {
    let conn = db_connection.lock().await;
    conn.call(|conn| match conn.execute(CREATE_TABLE, []) {
        Ok(_) => Ok(()),
        Err(e) => Err(tokio_rusqlite::Error::Rusqlite(e)),
    })
    .await
}

// Note that session_id is primary key and should be returned to caller via conn.last_insert_rowid()
// It's weird that sqlite::rowdata is 0'based (i.e. col1=row.get(0)?;), yet statements are 1'based...
// IN1: state_token (TEXT UNIQUE)
// IN2: client_address (TEXT)
// IN3: client_port (INTEGER)
// IN4: client_email (optional TEXT)
// IN5: access_token (TEXT (from server))
// IN6: refresh_token (optional TEXT)
// IN7: expires_in: (INTEGER (from server))
// IN8: expiry_time (INTEGER (epoch time))
const INSERT_TOKEN: &str = r#"
INSERT INTO tokens (
        state_token,
        client_address, client_port, client_email, 
        access_token, refresh_token, expires_in, expiry_time) 
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
"#;
pub(crate) async fn store_token(
    db_connection: &TDBConnectionLock_sqlite,
    token_data_no_session_id_ref: &TokenData,
) -> tokio_rusqlite::Result<i64> {
    let conn = db_connection.lock().await;
    let token_data_no_session_id = token_data_no_session_id_ref.clone();
    conn.call(move |conn| {
        conn.execute(
            INSERT_TOKEN,
            params![
                token_data_no_session_id.state_token,                      // 1
                token_data_no_session_id.client_address.to_string(),       // 2
                token_data_no_session_id.client_port,                      // 3
                token_data_no_session_id.possible_client_email,            // 4
                token_data_no_session_id.access_token,                     // 5
                token_data_no_session_id.possible_refresh_token,           // 6
                token_data_no_session_id.expires_in,                       // 7
                (token_data_no_session_id.expiry_time_as_sec_from_epoch() as i64), // 8
            ],
        )?;
        let session_id = conn.last_insert_rowid();
        Ok(session_id)
    })
    .await
}

// IN1: session_id (INTEGER)
const SELECT_TOKEN: &str = r#"
SELECT state_token, 
        client_address, client_port, client_email, 
        access_token, refresh_token, expires_in, expiry_time 
    FROM tokens WHERE session_id = ?1
"#;
pub(crate) async fn get_token_by_session_id(
    db_connection: &TDBConnectionLock_sqlite,
    session_id: u64,
) -> tokio_rusqlite::Result<TokenData> {
    let conn = db_connection.lock().await;
    conn.call(move |conn| {
        let mut stmt = conn.prepare(SELECT_TOKEN)?;
        // in parm1: session_id
        let token_data = stmt.query_row(params![session_id], |row| {
            Ok(TokenData::new(
                SessionIDType::ID(session_id),
                row.get(0)?,
                row.get::<usize, String>(1)?.to_string().parse().unwrap(),
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                UNIX_EPOCH + std::time::Duration::new(row.get(7)?, 10),
            ))
        })?;
        Ok(token_data)
    })
    .await
}
