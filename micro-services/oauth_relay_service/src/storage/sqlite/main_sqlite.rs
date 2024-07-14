use super::{storage_sqlite, TDBConnectionLock_sqlite};
use crate::{
    config::Config,
    messenger::TMQConnectionLock,
    web::actix::{keepalive, login},
};
use actix_web::{web, App, HttpServer};
use anyhow::Result as AnyResult;

const DB_PATHS: &str = "db/sqlite/tokens.sqlite";
const HTTP_LISTEN_ADDR: &str = "0.0.0.0:8080";

#[actix_web::main]
pub async fn sqlite_actix_main(
    config: &Config,
    db_connection: &TDBConnectionLock_sqlite,
    mq_connection: &TMQConnectionLock,
) -> AnyResult<(), std::io::Error> {
    // create DB table in case it does not exist yet
    storage_sqlite::create_table_token(db_connection)
        .await
        .unwrap();

    let db_connection_as_data = web::Data::new(db_connection.clone()); // cloning an Arc<T> just means incrementing the reference count
    let mq_connection_as_data = web::Data::new(mq_connection.clone());
    let config_as_data = web::Data::new(config.clone());
    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(db_connection_as_data.clone())
            .app_data(mq_connection_as_data.clone())
            .app_data(config_as_data.clone())
            .service(login::login)
            .service(keepalive::keepalive)
    })
    .bind(HTTP_LISTEN_ADDR)
    .unwrap()
    .run()
    .await;

    match http_server {
        Ok(_) => {
            println!("HTTP server started at {}", HTTP_LISTEN_ADDR);
        }
        Err(e) => {
            println!("Error starting HTTP server: {:?}", e);
        }
    }
    return Ok(());
}
