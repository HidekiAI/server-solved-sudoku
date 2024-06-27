use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

mod config;
mod tcp_handler;

type SharedTokens = Arc<Mutex<HashMap<String, TokenData>>>;

#[derive(Serialize, Deserialize)]
struct TokenData {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
    expiry_time: SystemTime,
}

#[derive(Deserialize)]
struct AuthRequest {
    code: String,
}

#[derive(Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
    refresh_token: Option<String>,
    scope: String,
    id_token: String,
}

async fn main() -> std::io::Result<()> {
    main_sqlite();
}

///////////////////////////////////////////////

async fn auth_callback_redis(
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
            let expiry_time = SystemTime::now() + Duration::from_secs(token_response.expires_in as u64);
            let token_data = TokenData {
                access_token: token_response.access_token,
                refresh_token: token_response.refresh_token,
                expires_in: token_response.expires_in,
                expiry_time,
            };
            // Store token data in Redis
            store_token(&redis_client, &auth_code, &token_data).await.unwrap();
            HttpResponse::Ok().json(token_response)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_web::main]
async fn main_redis() -> std::io::Result<()> {
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
            .route("/auth/callback", web::get().to(auth_callback_redis(query, config, client, redis_client, tokens)))
    })
    .bind("0.0.0.0:8080")?
    .run();

    let tcp_server = tcp_handler::start_tcp_server_redis(tokens.clone(), redis_client.clone());

    tokio::try_join!(http_server, tcp_server)?;

    Ok(())
}

//////////////////////////////////////////////////////////////

async fn auth_callback_postgres(
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
            let expiry_time = SystemTime::now() + Duration::from_secs(token_response.expires_in as u64);
            let token_data = TokenData {
                access_token: token_response.access_token,
                refresh_token: token_response.refresh_token,
                expires_in: token_response.expires_in,
                expiry_time,
            };
            // Store token data in PostgreSQL
            store_token(&pg_client, &auth_code, &token_data).await.unwrap();
            HttpResponse::Ok().json(token_response)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_web::main]
async fn main_postgres() -> std::io::Result<()> {
    let config = Config::from_env();
    let client = reqwest::Client::new();
    let (pg_client, pg_connection) = Config::new()
        .host("localhost")
        .user("user")
        .password("password")
        .dbname("db_name")
        .connect(NoTls)
        .await?;
    tokio::spawn(async move {
        if let Err(e) = pg_connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    let tokens: SharedTokens = Arc::new(Mutex::new(HashMap::new()));

    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(client.clone()))
            .app_data(web::Data::new(pg_client.clone()))
            .app_data(web::Data::new(tokens.clone()))
            .route("/auth/callback", web::get().to(auth_callback_postgres(query, config, client, pg_client, tokens)))
    })
    .bind("0.0.0.0:8080")?
    .run();

    let tcp_server = tcp_handler::start_tcp_server_postgres(tokens.clone(), pg_client.clone());

    tokio::try_join!(http_server, tcp_server)?;

    Ok(())
}



//////////////////////////////////////////////////////////////

async fn auth_callback_sqlite(
    query: web::Query<AuthRequest>,
    config: web::Data<Config>,
    client: web::Data<reqwest::Client>,
    conn: web::Data<Connection>,
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
            // Store token data in SQLite3
            store_token(&conn, &auth_code, &token_data).unwrap();
            HttpResponse::Ok().json(token_response)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}


async fn main_sqlite() -> std::io::Result<()> {
    let config = Config::from_env();
    let client = reqwest::Client::new();
    let conn = Connection::open("tokens.db").unwrap();
    let tokens: SharedTokens = Arc::new(Mutex::new(HashMap::new()));

    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(client.clone()))
            .app_data(web::Data::new(conn.clone()))
            .app_data(web::Data::new(tokens.clone()))
            .route("/auth/callback", web::get().to(auth_callback_sqlite(query, config, client, conn, tokens)))
    })
    .bind("0.0.0.0:8080")?
    .run();

    let tcp_server = tcp_handler::start_tcp_server_sqlite(tokens.clone(), conn.clone());

    tokio::try_join!(http_server, tcp_server)?;

    Ok(())
}
