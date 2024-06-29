use super::data_redis::{AuthRequest, SharedTokens, TokenData, TokenResponse};
use super::storage_redis;
use super::tcp_handler_redis as tcp_handler;
use crate::config::Config;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use redis::Client as RedisClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

async fn auth_callback(
    query: web::Query<AuthRequest>,
    config: web::Data<Config>,
    client: web::Data<Client>,
    redis_client: web::Data<RedisClient>,
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
            // Store token data in Redis
            storage_redis::store_token(&redis_client, &auth_code, &token_data)
                .await
                .unwrap();
            HttpResponse::Ok().json(token_response)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_env();
    let client = Client::new();
    let redis_client = RedisClient::open("redis://127.0.0.1/").unwrap();
    let tokens: SharedTokens = Arc::new(Mutex::new(HashMap::new()));

    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(client.clone()))
            .app_data(web::Data::new(redis_client.clone()))
            .app_data(web::Data::new(tokens.clone()))
            .route("/auth/callback", web::get().to(auth_callback))
    })
    .bind("0.0.0.0:8080")?
    .run();

    let tcp_server = tcp_handler::start_tcp_server(tokens.clone(), redis_client.clone());

    tokio::try_join!(http_server, tcp_server)?;

    Ok(())
}
