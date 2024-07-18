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
    let current_dir = std::env::current_dir().unwrap();
    let env_file_path = current_dir.join("build/.env");
    let config = Config::from_env_paths(env_file_path.as_path());
    let db_connection = storage::open_db_connection_from_config(config.clone()).await;
    let mq_connection = messenger::open_mq_connection_from_config(config.clone()).await;
    sqlite_actix_main(&config, &db_connection, &mq_connection)
}
