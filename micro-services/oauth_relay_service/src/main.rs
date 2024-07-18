//#include modules:
pub mod config;
pub mod data;
pub mod messenger;
pub mod storage;
pub mod web;

use std::{env, path::Path};

use config::Config;
use storage::sqlite::main_sqlite::sqlite_actix_main;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_local_env_file();
    let db_connection = storage::open_db_connection_from_config(&config).await;
    let (mq_producer, mq_consumer) = messenger::open_mq_connections_from_config(&config).await;
    sqlite_actix_main(&config, &db_connection, &mq_producer, &mq_consumer)
}
