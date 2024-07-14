use crate::{
    data::{KeepaliveRequest, KeepaliveResponse, TokenData},
    storage::{self, TDBConnectionLock},
    web::web_consts::*,
};
use actix_web::{web, HttpRequest, HttpResponse};
use serde_json;
use serde_urlencoded;
use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// Keep-alive route - Check if the client is authenticated and handle keep-alive
/// HTTP verb: GET
/// params: last_session_id
#[actix_web::get("/keepalive")]
pub async fn keepalive(
    client_http_request: HttpRequest,
    db_connection: web::Data<TDBConnectionLock>,
) -> HttpResponse {
    let config = crate::config::Config::from_env();
    let query_string = client_http_request.query_string();
    let query_params: HashMap<String, String> =
        serde_urlencoded::from_str(query_string).unwrap_or_else(|_| HashMap::new());
    let keep_alive_request = KeepaliveRequest {
        last_session_id: query_params
            .get("last_session_id")
            .expect("last_session_id not found")
            .clone(),
    };

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
    let possible_session_data: Option<TokenData> = storage::get_token_by_session_id(
        &config,
        db_connection.as_ref(),
        &Some(keep_alive_request.last_session_id),
    )
    .await;
    match possible_session_data {
        Some(token_data) => {
            // if result row-set exists, count SHOULD be 1 (cannot have more than 1 UNIQUE key)
            println!(
                "Keep-alive: Found session data for session_id: {:?}",
                token_data.session_id()
            );
            let update =
                storage::upsert_token_data(&config, db_connection.as_ref(), &token_data)
                    .await;
            match update {
                Ok(_) => {
                    println!(
                        "Keep-alive: Updated last_accessed for session_id: {:?}",
                        token_data.session_id()
                    );
                    let next_update = SystemTime::now()
                        .checked_add(TOKEN_REFRESH_INTERVAL)
                        .unwrap()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    match token_data.is_expired() {
                        false => {
                            println!(
                                "Keep-alive: Token is not expired for session_id: {:?}",
                                token_data.session_id()
                            );
                            let response_body = serde_json::to_string(&KeepaliveResponse {
                                next_expected_time: next_update,
                                ttl: TOKEN_REFRESH_INTERVAL.as_secs(),
                                status: "OK".to_string(),
                                message: "Keep-alive successful".to_string(),
                            })
                            .unwrap();
                            return HttpResponse::Ok().body(response_body);
                        }
                        true => {
                            println!(
                                "Keep-alive: Token is expired for session_id: {:?}",
                                token_data.session_id()
                            );
                            todo!("Handle expired token");
                        }
                    }
                }
                Err(e) => {
                    println!("Keep-alive: Failed to update last_accessed for session_id: {:?} with error: {:?}", token_data.session_id(), e);
                }
            }
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
async fn health_checker_handler() -> HttpResponse {
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
