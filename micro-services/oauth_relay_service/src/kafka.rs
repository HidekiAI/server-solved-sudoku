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

pub type TMQConnection = Arc<Mutex<kafka>>;

fn open_mq_connection_from_config(config: Config) -> TMQConnection {
    match config.mq_connection {
        MQType::Kafka {
            host_address,
            host_port,
        } => open_mq_connection(host_address, host_port),
        _ => panic!("Unsupported MQType"),
    }
}
