//#include
mod data_sqlite;
mod handlers_sqlite;
mod main_sqlite;
mod storage_sqlite;

use std::sync::Arc;
use anyhow::Result as AnyResult;

use data_sqlite::*;
use handlers_sqlite::*;
use main_sqlite::*;
use storage_sqlite::*;
use tokio::sync::Mutex;

pub type TDBConnection = Arc<Mutex<tokio_rusqlite::Connection>>;
pub fn open_db_connection(db_path: &str) -> AnyResult<TDBConnection> {
    let result = tokio_rusqlite::Connection::open(db_path);
    match result {
        Ok(db_connection) => Ok(new_connection(db_connection)),
        Err(e) => panic!("Error connecting to database: '{}'", e.map_err(|e| e.into())),
    }
}
fn new_connection(db_connection: tokio_rusqlite::Connection) -> TDBConnection {
    Arc::new(Mutex::new(db_connection))
}
