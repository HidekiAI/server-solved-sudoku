//#include
mod data_sqlite;
mod handlers_sqlite;
mod main_sqlite;
mod storage_sqlite;

use anyhow::Result as AnyResult;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::*;
use data_sqlite::*;
use handlers_sqlite::*;
use main_sqlite::*;
use storage_sqlite::*;

pub type TDBConnection = Arc<Mutex<tokio_rusqlite::Connection>>;

pub async fn open_db_connection(db_path: &str) -> AnyResult<TDBConnection> {
    let result = tokio_rusqlite::Connection::open(db_path).await;
    match result {
        Ok(db_connection) => Ok(new_connection(db_connection)),
        Err(e) => panic!("Error connecting to database: '{}'", e),
    }
}
fn new_connection(db_connection: tokio_rusqlite::Connection) -> TDBConnection {
    Arc::new(Mutex::new(db_connection))
}

async fn open_db_connection_from_config(config: Config) -> TDBConnection {
    match config.db_connection {
        DBType::SQLite { db_path } => open_db_connection(db_path.as_str()).await.unwrap(),
        _ => panic!("Unsupported DBType"),
    }
}
