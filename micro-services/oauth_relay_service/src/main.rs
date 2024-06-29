use super::config::Config;
use super::postgres;
use super::redis;
use super::sqlite;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use config::Config;
use redis::{self, Client};
use reqwest::Client;
use rusqlite::{self, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio_postgres::{self, Client};

// include files:
mod config;
mod postgres;
mod redis;
mod sqlite;
mod tcp_handler;

async fn main() -> std::io::Result<()> {
    sqlite::main_sqlite::main()
}
