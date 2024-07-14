//#include
pub mod data_sqlite;
pub mod handlers_sqlite;
pub mod main_sqlite;
pub mod storage_sqlite;

use anyhow::Result as AnyResult;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type TDBConnection_sqlite = tokio_rusqlite::Connection; // currently using tokio-rusqlite
pub type TDBConnectionLock_sqlite = Arc<Mutex<TDBConnection_sqlite>>;

pub async fn open_db_connection_rusqlite(db_path: &str) -> AnyResult<TDBConnectionLock_sqlite> {
    let result = tokio_rusqlite::Connection::open(db_path).await;
    match result {
        Ok(db_connection) => Ok(new_connection_sqlite(db_connection)),
        Err(e) => panic!("Error connecting to database: '{}'", e),
    }
}

fn new_connection_sqlite(db_connection: tokio_rusqlite::Connection) -> TDBConnectionLock_sqlite {
    Arc::new(Mutex::new(db_connection))
}

pub(crate) async fn get_token_by_session_id(
    db_connection: &TDBConnectionLock_sqlite,
    last_session_id: &Option<String>,
) -> Option<crate::data::TokenData> {
    todo!()
}

pub(crate) async fn upsert_token_data(
    db_connection: &TDBConnectionLock_sqlite,
    token_data: &crate::data::TokenData,
) -> Result<(), anyhow::Error> {
    todo!()
}

//async fn open_db_connection_from_config_sqlite(config: Config) -> TDBConnectionLock_sqlite {
//    match config.db_connection {
//        DBType::SQLite { db_path } => open_db_connection_rusqlite(db_path.as_str()).await.unwrap(),
//        _ => panic!("Unsupported DBType"),
//    }
//}
