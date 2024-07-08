use super::storage_sqlite;
use crate::{
    config::Config,
    data::{OAuth2AuthCodeRequest, OAuth2TokenResponse, SessionIDType, TokenData},
    sqlite::{open_db_connection, TDBConnection},
};
use actix_web::{get, http::ConnectionType, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result as AnyResult;
use std::time::{Duration, SystemTime};
use tokio_rusqlite::Connection;

const DB_PATHS: &str = "db/sqlite/tokens.sqlite";
const HTTP_LISTEN_ADDR: &str = "0.0.0.0:8080";

#[actix_web::main]
pub async fn sqlite_actix_main() -> AnyResult<(), String> {
    let possible_db_connection_raw = open_db_connection(DB_PATHS).await;
    match possible_db_connection_raw {
        Ok(db_connection) => {
            // create DB table in case it does not exist yet
            storage_sqlite::create_table_token(db_connection.clone())
                .await
                .unwrap();

            let env_config = Config::from_env();
            let http_requesting_client = reqwest::Client::new();
            let http_server = HttpServer::new(move || {
                let db_connection_as_data = web::Data::new(db_connection.clone()); // cloning an Arc<T> just means incrementing the reference count
                App::new()
                    .app_data(db_connection_as_data)
                    .route("/login", web::get().to(crate::sqlite::login))
                    .route("/keepalive", web::get().to(crate::sqlite::keepalive))
            })
            .bind(HTTP_LISTEN_ADDR)
            .unwrap()
            .run()
            .await;

            return Ok(());
        }
        Err(e) => {
            return Err(format!("Error connecting to database: '{}'", e));
        }
    }
}
