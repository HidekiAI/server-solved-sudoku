use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use config::Config;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc; // we're using this ARC
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex; // Note: We're using this Mutex rather than std::sync::Mutex
use tokio_rusqlite::Connection;

mod config;
mod data;
mod sqlite;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
   crate::sqlite::main_sqlite::sqlite_actix_main().await
}
