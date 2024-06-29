use super::data_postgres::{AuthRequest, SharedTokens, TokenData, TokenResponse};
use super::{storage_postgres, tcp_handler_postgres as tcp_handler};
use crate::config::Config;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio_postgres::{Client as PgClient, NoTls};

async fn auth_callback(
    query: web::Query<AuthRequest>,
    config: web::Data<Config>,
    client: web::Data<reqwest::Client>,
    pg_client: web::Data<PgClient>,
    tokens: web::Data<SharedTokens>,
) -> impl Responder {
    let auth_code = query.into_inner().code;
    let token_url = "https://oauth2.googleapis.com/token";

    let params = [
        ("code", auth_code.as_str()),
        ("client_id", config.google_client_id.as_str()),
        ("client_secret", config.google_client_secret.as_str()),
        ("redirect_uri", config.google_redirect_uri.as_str()),
        ("grant_type", "authorization_code"),
    ];

    let response = client.post(token_url).form(&params).send().await;

    match response {
        Ok(resp) => {
            let token_response: TokenResponse = resp.json().await.unwrap();
            let expiry_time =
                SystemTime::now() + Duration::from_secs(token_response.expires_in as u64);
            let token_data = TokenData {
                access_token: token_response.access_token,
                refresh_token: token_response.refresh_token,
                expires_in: token_response.expires_in,
                expiry_time,
            };
            // Store token data in PostgreSQL
            storage_postgres::store_token(&pg_client, &auth_code, &token_data)
                .await
                .unwrap();
            HttpResponse::Ok().json(token_response)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

//#[actix_web::main]
//async fn main_actix() -> std::io::Result<()> {
//    let config = Config::from_env();
//    let client = reqwest::Client::new();
//    let (pg_client, pg_connection) = Config::new()
//        .host("localhost")
//        .user("user")
//        .password("password")
//        .dbname("db_name")
//        .connect(NoTls)
//        .await?;
//    tokio::spawn(async move {
//        if let Err(e) = pg_connection.await {
//            eprintln!("connection error: {}", e);
//        }
//    });
//    let tokens: SharedTokens = Arc::new(Mutex::new(HashMap::new()));
//
//    let http_server = HttpServer::new(move || {
//        App::new()
//            .app_data(web::Data::new(config.clone()))
//            .app_data(web::Data::new(client.clone()))
//            .app_data(web::Data::new(pg_client.clone()))
//            .app_data(web::Data::new(tokens.clone()))
//            .route("/auth/callback", web::get().to(auth_callback))
//    })
//    .bind("0.0.0.0:8080")?
//    .run();
//
//    let tcp_server = tcp_handler::start_tcp_server(tokens.clone(), pg_client.clone());
//
//    tokio::try_join!(http_server, tcp_server)?;
//
//    Ok(())
//}

async fn handle_oauth_callback(// Define necessary parameters and extract tokens from the callback
) -> impl Responder {
    // Handle the OAuth2 callback, extract tokens and store them
    "OAuth2 callback handled"
}

#[tokio::main]
async fn main_tokio() {
    // Initialize your tokens and PostgreSQL client
    let tokens: SharedTokens = Arc::new(Mutex::new(HashMap::new()));

    let (pg_client, connection) =
        tokio_postgres::connect("host=localhost user=postgres", tokio_postgres::NoTls)
            .await
            .unwrap();

    // Spawn the connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Start the TCP server in a separate task
    tokio::spawn(start_tcp_server(tokens.clone(), pg_client));

    // Start the Actix-web server
    use actix_web::{web, App, HttpServer, Responder};

    async fn handle_oauth_callback() -> impl Responder {
        // Handle the OAuth2 callback, extract tokens and store them
        "OAuth2 callback handled"
    }

    HttpServer::new(move || {
        App::new()
            .route("/oauth2/callback", web::get().to(handle_oauth_callback))
            .app_data(web::Data::new(tokens.clone()))
    })
    .bind("0.0.0.0:8080")
    .unwrap()
    .run()
    .await
    .unwrap();
}
