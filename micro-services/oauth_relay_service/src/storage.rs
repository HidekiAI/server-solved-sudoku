//#include
pub(crate) mod sqlite;

use anyhow::Result as AnyResult;
use core::panic;
use std::{any::Any, net::IpAddr, sync::Arc};
use tokio::sync::Mutex;

use crate::{
    config::*,
    data::{SessionIDType, TokenData},
};

pub type TDBConnection = sqlite::TDBConnection_sqlite; // currently using tokio-rusqlite
pub type TDBConnectionLock = sqlite::TDBConnectionLock_sqlite;

fn new_connection(db_connection: TDBConnection) -> TDBConnectionLock {
    Arc::new(Mutex::new(db_connection))
}

pub async fn open_db_connection_from_config(config: Config) -> TDBConnectionLock {
    match config.db_connection {
        DBType::SQLite { db_path } => sqlite::open_db_connection_rusqlite(db_path.as_str())
            .await
            .unwrap(),
        DBType::PostgresSQL {
            host_address: _,
            host_port: _,
        } => panic!("PostgreSQL not yet implemented"),
        //_ => panic!("Unsupported DBType"),
    }
}

pub(crate) fn get_db_port(config: &Config) -> Option<u16> {
    match config.db_connection {
        DBType::SQLite { db_path: _ } => None,
        DBType::PostgresSQL {
            host_address: _,
            host_port,
        } => Some(host_port),
    }
}

pub(crate) fn get_db_address(config: &Config) -> Option<IpAddr> {
    match config.db_connection {
        DBType::SQLite { db_path: _ } => None,
        DBType::PostgresSQL {
            host_address,
            host_port: _,
        } => Some(host_address),
    }
}

pub(crate) fn get_db_path(config: &Config) -> Option<String> {
    match &config.db_connection {
        DBType::SQLite { db_path } => Some(db_path.clone()),
        DBType::PostgresSQL {
            host_address: _,
            host_port: _,
        } => None,
    }
}

pub(crate) fn get_db_type(config: &Config) -> String {
    match config.db_connection {
        DBType::SQLite { db_path: _ } => "sqlite".to_string(),
        DBType::PostgresSQL {
            host_address: _,
            host_port: _,
        } => "postgres".to_string(),
    }
}

pub(crate) async fn get_token_by_session_id(
    config: &Config,
    db_connection: &TDBConnectionLock,
    last_session_id: &Option<String>,
) -> Option<TokenData> {
    //    db_connection
    //        .lock()
    //        .await
    //        .call(move |conn| {
    //            conn.query_row(
    //                "SELECT COUNT(*) FROM tokens WHERE session_id = ?",
    //                params![last_session_id],
    //                |row| row.get(0),
    //            )
    //            .map_err(|e| tokio_rusqlite::Error::from(e))
    //        })
    match config.db_connection {
        DBType::SQLite { db_path: _ } => {
            sqlite::get_token_by_session_id(db_connection, last_session_id).await
        }
        DBType::PostgresSQL {
            host_address: _,
            host_port: _,
        } => panic!("PostgreSQL not yet implemented"),
    }
}

pub(crate) async fn upsert_token_data(
    config: &Config,
    db_connection: &TDBConnectionLock,
    token_data: &TokenData,
) -> AnyResult<()> {
    match config.db_connection {
        DBType::SQLite { db_path: _ } => sqlite::upsert_token_data(db_connection, token_data).await,
        DBType::PostgresSQL {
            host_address: _,
            host_port: _,
        } => panic!("PostgreSQL not yet implemented"),
    }
}
