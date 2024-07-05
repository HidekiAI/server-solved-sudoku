use actix_session::Session;
use actix_web::{
    body::MessageBody,
    http::{header, StatusCode},
    web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    Scope, TokenUrl,
};
use reqwest::Response;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_urlencoded;
use std::{
    borrow::Borrow,
    collections::HashMap,
    net::IpAddr,
    ops::Deref,
    str::FromStr,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::{broadcast, futures, mpsc::{channel, Receiver, Sender}, Mutex};
use tokio_rusqlite::{params, Connection};

use crate::{
    config::Config,
    data::{
        OAuth2AuthCodeRequest, OAuth2AuthCodeResponse, OAuth2TokenResponse, SessionIDType,
        TokenData,
    },
};

use super::{storage_sqlite, store_token};

const TOKEN_REFRESH_INTERVAL: Duration = Duration::from_secs(3600);
const TOKEN_REFRESH_INTERVAL_MARGIN: Duration = Duration::from_secs(30);

// From https://accounts.google.com/.well-known/openid-configuration:
//    "issuer": "https://accounts.google.com",
//    "authorization_endpoint": "https://accounts.google.com/o/oauth2/v2/auth",
//    "device_authorization_endpoint": "https://oauth2.googleapis.com/device/code",
//    "token_endpoint": "https://oauth2.googleapis.com/token",
//    "userinfo_endpoint": "https://openidconnect.googleapis.com/v1/userinfo",
//    "revocation_endpoint": "https://oauth2.googleapis.com/revoke",
const AUTH_URL_GET: &str = "https://accounts.google.com/o/oauth2/v2/auth"; // GET
const TOKEN_URL_POST: &str = "https://oauth2.googleapis.com/token"; // POST
const USER_INFO_URL_GET: &str = "https://openidconnect.googleapis.com/v1/userinfo"; // GET

pub type TConnection = Arc<Mutex<tokio_rusqlite::Connection>>;
pub fn new_connection(db_connection: tokio_rusqlite::Connection) -> TConnection {
    Arc::new(Mutex::new(db_connection))
}

// serde_json deserialization (for GET request)
#[derive(Deserialize)]
struct LoginRequest {
    last_session_id: Option<String>, // used for recovery/reconnect/resume
}
// serde_json serialization (for response)
#[derive(Serialize)]
struct LoginResponse {
    session_id: String,
    status: String,
    message: String,
}

#[derive(Deserialize)]
struct KeepaliveRequest {
    last_session_id: String,
}
#[derive(Serialize)]
struct KeepaliveResponse {
    next_expected_time: u64, // absolute time, EPOCH based
    // time-to-live in seconds (problem with relative deltaT is that if round-trip takes long time,
    // few seconds may have passed already...) probably better to just calcuate
    // TTL = next_expected_time - current_time on client side...
    ttl: u64,
    status: String,
    message: String,
}

// HTTP GET to request Google OAuth2 authorization code (AUTH_URL_GET)
// sample request:
//   https://accounts.google.com/o/oauth2/v2/auth?
//    scope=https%3A//www.googleapis.com/auth/drive.metadata.readonly&
//    access_type=offline&
//    include_granted_scopes=true&
//    response_type=code&
//    state=state_parameter_passthrough_value&
//    redirect_uri=https%3A//hostname.mydomain.tld/auth_code_callback&
//    client_id=client_id
async fn request_auth_code(
    config: web::Data<Config>,
    http_client: web::Data<reqwest::Client>,
    conn_as_data: web::Data<TConnection>,
) -> Result<Response, reqwest::Error> {
    let auth_request = OAuth2AuthCodeRequest {
        client_id: config.google_client_id.clone(),
        redirect_uri: config.google_redirect_uri.clone(),
        response_type: "code".to_string(),
        scope: "https://www.googleapis.com/auth/drive.metadata.readonly".to_string(),
        access_type: Some("offline".to_string()),
        state: Some("state_parameter_passthrough_value".to_string()),
        include_granted_scopes: Some(true),
        prompt: Some("consent select_account".to_string()),
    };
    let response = http_client
        .get(AUTH_URL_GET)
        .query(&[
            &format!("client_id={}", auth_request.client_id),
            &format!("redirect_uri={}", auth_request.redirect_uri),
            &format!("response_type={}", auth_request.response_type),
            &format!("scope={}", auth_request.scope),
            &format!(
                "access_type={}",
                auth_request.access_type.unwrap_or_default()
            ),
            &format!("state={}", auth_request.state.unwrap_or_default()),
            &format!(
                "include_granted_scopes={}",
                auth_request.include_granted_scopes.unwrap_or_default()
            ),
            &format!("prompt={}", auth_request.prompt.unwrap_or_default()),
        ])
        .send()
        .await;
    response
}

// This callback is triggered based off of the service requesting AuthorizationEndpoint (AUTH_URL_GET)
// Though I've documented via UML, the flow is as follows:
// 1. Service requests AUTH_URL_GET (Google OAuth2 AuthorizationEndpoint)
// 2. Google OAuth2 AuthorizationEndpoint will negotiate with client and based on prompt (consent select_account), client will consent
// 3. Upon consentment, we'll get a callback here with either error or code
// 4. If we received a valid auth-code, we'll use that to request Google OAuth2 Token (TOKEN_URL_POST) using THIS endpoint as redirect_uri (claiming "I'm the caller that you called back to")
// 5. Upon successful token request, we'll get back a token (access_token, refresh_token, expires_in, etc.) which we'll store in SQLite3
// 6. We'll then use the access_token to request user's email address (USER_INFO_URL_GET)
// 7. We'll then return a session_id (which is a UUID) to the client
// sample CALLBACK response from request to AUTH_URL_GET
//     https://hostname.mydomain.tld/auth_code_callback?error=access_denied
//     https://hostname.mydomain.tld/auth_code_callback?code=4/P7q7W91a-oMsCeLvIaQm6bTrgtp7
#[actix_web::get("/auth_callback")] // MUST match Config::google_redirect_uri! (actually it's GOOGLE_REDIRECT_URI in .env file)
pub async fn auth_code_callback(
    client_http_request: HttpRequest, // unfortunately, for callback from external, we only get this and body (most likely empty)
) -> impl Responder {
    let query_string = client_http_request.query_string();
    let query_params: HashMap<String, String> =
        serde_urlencoded::from_str(query_string).unwrap_or_else(|_| HashMap::new());
    // we don't really need IP of Google Service, but for debugging/logging purposes, we'll keep it
    // NOTE: Don't log query_params, for it contains auth code...
    let client_ip = client_http_request
        .peer_addr()
        .map(|addr| addr.ip())
        .unwrap();
    let client_port = client_http_request
        .peer_addr()
        .map(|addr| addr.port())
        .unwrap();
    println!(
        "AuthCodeCallback: Client (Google Cloud) IP={} (port={})",
        client_ip, client_port
    );

    let auth_code_response = OAuth2AuthCodeResponse {
        error: query_params.get("error").map(|s| s.to_string()),
        code: query_params.get("code").map(|s| s.to_string()),
    };
    match auth_code_response.error {
        Some(_) => {
            // Error occurred, let's log it and return 500
            println!(
                "AuthCodeCallback: Error={}",
                auth_code_response.error.unwrap()
            );
            HttpResponse::InternalServerError().finish()
        }
        None => {
            // First, if it is NOT an error, let's go ahead and request OAuth2Token from Google
            let auth_code = auth_code_response.code.unwrap(); // should panic if code is not present!
            let token_response =
                request_and_store_google_oauth2(config, http_client, conn_as_data).await;

            // Now, query for user's email client_address
            match token_response {
                Ok(resp) => {
                    // deserialize the response from Google OAuth2
                    let oauth2_token_response: OAuth2TokenResponse = resp.json().await.unwrap();
                    let http_client = reqwest::Client::new();
                    let user_info_response = http_client.get(USER_INFO_URL_GET).header(
                        "Authorization",
                        format!("Bearer {}", oauth2_token_response.access_token),
                    );

                    // wrap LoginResponse in a Result (Body)
                    let response_body = serde_json::to_string(&LoginResponse {
                        session_id: "1234567890".to_string(),
                        status: "OK".to_string(),
                        message: "Login successful".to_string(),
                    })
                    .unwrap();
                    HttpResponse::Ok().body(response_body)
                }
                Err(_) => HttpResponse::InternalServerError().finish(),
            }

            // TODO: signal (via kafka) of new session_id (new login) for any services who cares for that event...
        }
    }
}

// HTTP POST to request Google OAuth2 token (TOKEN_URL_POST)
// this method is called from auth_code_callback() to get the token from Google
async fn request_and_store_google_oauth2(
    query: web::Query<OAuth2AuthCodeRequest>,
    config: web::Data<Config>,
    http_client: web::Data<reqwest::Client>,
    conn_as_data: web::Data<TConnection>,
) -> impl Responder {
    let auth_code = query.into_inner().code;

    let params = [
        ("code", auth_code.as_str()),
        ("client_id", config.google_client_id.as_str()),
        ("client_secret", config.google_client_secret.as_str()),
        ("redirect_uri", config.google_redirect_uri.as_str()),
        ("grant_type", "authorization_code"),
    ];

    let response = http_client.post(TOKEN_URL_POST).form(&params).send().await;
    match response {
        Ok(resp) => {
            // deserialize the response from Google OAuth2
            let oauth2_token_response: OAuth2TokenResponse = resp.json().await.unwrap();

            let expiry_time =
                SystemTime::now() + Duration::from_secs(oauth2_token_response.expires_in as u64);
            let token_data_no_client_info = TokenData::new(
                SessionIDType::Undefined(None), // when we get back response from Google, we have no SessionID...
                IpAddr::from_str("::1").unwrap(),
                0,
                None,
                oauth2_token_response.access_token, // The token that your application sends to authorize a Google API request.
                // A token that you can use to obtain a new access token.
                // Refresh tokens are valid until the user revokes access.
                // Again, this field is only present in this response if
                // you set the access_type parameter to offline in the
                // initial request to Google's authorization server
                oauth2_token_response.refresh_token,
                oauth2_token_response.expires_in, // The remaining lifetime of the access token in seconds.
                expiry_time,
            );
            // Store token data in SQLite3 just in case something goes wrong...
            let conn: TConnection = conn_as_data.get_ref().clone();
            let ret_session_id = store_token(conn, token_data_no_client_info.clone())
                .await
                .unwrap();
            println!("Stored token with session_id={}", ret_session_id); // don't log access_token, instead log session_id
            let token_response = serde_json::to_string(&token_data_no_client_info).unwrap();
            HttpResponse::Ok().json(token_response)
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// this method is indirectly called from auth_code_callback() (chained call when qurest_and_store_google_oauth2 succeeds)
// get the user's email address from Google
async fn request_userinfo(
    token: String,
    http_client: web::Data<reqwest::Client>,
) -> impl Responder {
    // once we got OK/200 from Google OAuth2, get e-mail address (make sure Google API was setup with email priv enabled)
    // Note that www.googleapis.com
    // $curl -X GET "https://www.googleapis.com/oauth2/v1/userinfo?alt=json" -H"Authorization: Bearer accessTokenHere"
    let user_info_response = http_client
        .get(USER_INFO_URL_GET)
        .header("Authorization", format!("Bearer {}", token));

    // wrap LoginResponse in a Result (Body)
    let response_body = serde_json::to_string(&LoginResponse {
        session_id: "1234567890".to_string(),
        status: "OK".to_string(),
        message: "Login successful".to_string(),
    })
    .unwrap();
    HttpResponse::Ok().body(response_body)
}

/// Login route - either authenticate or re-authenticate the client against Google OAuth2
/// HTTP verb: GET
/// params: last_session_id (optional)
/// see: https://developers.google.com/static/identity/protocols/oauth2/images/flows/authorization-code.png
#[actix_web::get("/login")]
pub async fn login(
    client_http_request: HttpRequest,
    db_connection: web::Data<TConnection>,
) -> impl Responder {
    let query_string = client_http_request.query_string();
    let query_params: HashMap<String, String> =
        serde_urlencoded::from_str(query_string).unwrap_or_else(|_| HashMap::new());
    let login_request = LoginRequest {
        last_session_id: query_params.get("last_session_id").map(|s| s.to_string()),
    };

    // grab some data we want to setup TokenData (we SHOULD panic if we cannot get client IP/port)
    let client_ip = client_http_request
        .peer_addr()
        .map(|addr| addr.ip())
        .unwrap();
    let client_port = client_http_request
        .peer_addr()
        .map(|addr| addr.port())
        .unwrap();
    println!("Login: Client IP={} (port={})", client_ip, client_port);

    // I'd like to now block and wait for the signal that I've got a session_id...
    loop {

    }

    HttpResponse::Ok().body(response_body)
}

/// Keep-alive route - Check if the client is authenticated and handle keep-alive
/// HTTP verb: GET
/// params: last_session_id
#[actix_web::get("/keepalive")]
pub async fn keepalive(
    client_http_request: HttpRequest,
    db_connection: web::Data<TConnection>,
) -> impl Responder {
    let query_string = client_http_request.query_string();
    let query_params: HashMap<String, String> =
        serde_urlencoded::from_str(query_string).unwrap_or_else(|_| HashMap::new());
    let last_session_id = query_params.get("last_session_id").map(|s| s.to_string());

    // grab some data we want to setup TokenData
    let client_ip = client_http_request
        .peer_addr()
        .map(|addr| addr.ip())
        .unwrap();
    let client_port = client_http_request
        .peer_addr()
        .map(|addr| addr.port())
        .unwrap();

    // first check if sessionID is valid (exists in DB)
    let is_session_valid = db_connection
        .lock()
        .await
        .call(move |conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM tokens WHERE session_id = ?",
                params![last_session_id],
                |row| row.get(0),
            )
            .map_err(|e| tokio_rusqlite::Error::from(e))
        })
        .await;

    match is_session_valid {
        Ok(1) => {
            // if result row-set exists, count SHOULD be 1 (cannot have more than 1 UNIQUE key)
        }
        _ => {
            // REJECT/DROP this undesired client ASAP...
            panic!("Keep-alive: Could not get client IP/port");
        }
    }
    let ttl_in_seconds = Duration::from_secs(60);
    let next_expected_time_in_epoch = SystemTime::now()
        .checked_add(ttl_in_seconds)
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let response_body = serde_json::to_string(&KeepaliveResponse {
        next_expected_time: next_expected_time_in_epoch,
        ttl: ttl_in_seconds.as_secs(),
        status: "OK".to_string(),
        message: "Keep-alive successful".to_string(),
    })
    .unwrap();

    HttpResponse::Ok().body(response_body)
}

#[actix_web::get("/test/keepalive")]
async fn health_checker_handler() -> impl Responder {
    const MESSAGE1: &str = "OK";
    let duration = std::time::Duration::new(30, 0);
    let next_time = std::time::SystemTime::now()
        .checked_add(duration)
        .expect("Failed to calculate next time;")
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    let next_expected = format!("{}", next_time.as_secs());

    HttpResponse::Ok()
        .json(serde_json::json!({"status": "success", "ack": MESSAGE1, "next": next_expected}))
}
