use super::{storage_sqlite, store_token};
use crate::{
    config::Config,
    data::*,
};
use actix_web::{web, FromRequest, HttpRequest, HttpResponse, Responder};
use base64::{engine::general_purpose, Engine};
use rand::RngCore;
use reqwest::Response;
use serde_urlencoded;
use std::{
    collections::HashMap,
    net::IpAddr,
    str::FromStr,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex;
use tokio_rusqlite::params;

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

////////////////////////////////////////////////////////////////////////////////////
// Though I've documented via UML, the flow is as follows:
// 1. User login via /login (GET) which will trigger call to Google OAuth2 
//    AuthorizationEndpoint via request_auth_code_trigger_callback()
// 2. Service requests GET request to AUTH_URL_GET (Google OAuth2 AuthorizationEndpoint) 
//    with OAuth2AuthCodeRequest, and schedules for a callback (see auth_code_callback())
// 3. Google OAuth2 AuthorizationEndpoint will negotiate with client and based on 
//    prompt (consent select_account), client will consent
// 4. Upon consentment, we'll get a HTTP GET to callback auth_code_callback() with 
//    either error or auth-code on the URI parameter
// 5. auth_code_callback() - If we received a valid auth-code (in HTTP GET URI params), 
//    we'll use that to request Google OAuth2 Token (TOKEN_URL_POST) using THIS 
//    endpoint as redirect_uri (claiming "I'm the caller that you called back to") 
//     via request_and_store_google_oauth2()
// 6. auth_code_callback() - Upon successful token GET request, we'll get back a token
//    (access_token, refresh_token, expires_in, etc.) which we'll store in SQLite3
// 7. auth_code_callback() - We'll then use the access_token to request user's email 
//    address via HTTP GET to USER_INFO_URL_GET (within login() method)
// 8. auth_code_callback() - if all is well, we'll now signal/notify/message/publish
//    that we have a new session_id (new login) for any services who cares for that 
//    event...
// 9. We'll then return a session_id and state_token to the client (via /login, 
//    LoginResponse)
////////////////////////////////////////////////////////////////////////////////////

// HTTP GET to request Google OAuth2 authorization code (AUTH_URL_GET) via OAuth2AuthCodeRequest
// sample request:
//   https://accounts.google.com/o/oauth2/v2/auth?response_type=code
//      &client_id=my_client_id
//      &redirect_uri=my_redirect_uri
//      &scope=scope&state=my_state
//      &access_type=my_access_type
//      &include_granted_scopes=my_include_granted_scopes
//      &prompt=select_account%20consent
async fn request_auth_code_trigger_callback(
    query_get_auth_request: web::Query<OAuth2AuthCodeRequest>,
    http_client: web::Data<reqwest::Client>,
) -> Result<Response, reqwest::Error> {
    let response = http_client
        .get(AUTH_URL_GET)
        .query(&[
            &format!("client_id={}", query_get_auth_request.client_id),
            &format!("redirect_uri={}", query_get_auth_request.redirect_uri),
            &format!("response_type={}", query_get_auth_request.response_type),
            &format!("scope={}", query_get_auth_request.scope),
            &format!(
                "access_type={}",
                match query_get_auth_request.possible_access_type.clone() {
                    Some(access_type) => access_type,
                    None => "online".to_string(),
                }
            ),
            &format!("state={}", match query_get_auth_request.possible_state.clone() {
                Some(state) => state,
                None => "".to_string(),
            }),
            &format!(
                "include_granted_scopes={}",
                query_get_auth_request
                    .possible_include_granted_scopes
                    .unwrap_or_default()
            ),
            &format!(
                "prompt={}",
                match query_get_auth_request.possible_prompt.clone() {
                    Some(prompt) => prompt,
                    None => "consent".to_string(),
                }
            ),
        ])
        .send()
        .await;
    // Should be almost immediate, this request is to trigger callback in which
    // the user needs to consent/authorize, and once it's all good, we'll get a callback
    response
}


// This callback is triggered based off of the service requesting AuthorizationEndpoint (AUTH_URL_GET)
// sample CALLBACK response from request to AUTH_URL_GET
//     https://hostname.mydomain.tld/auth_callback?error=access_denied
//     https://hostname.mydomain.tld/auth_callback?code=4/P7q7W91a-oMsCeLvIaQm6bTrgtp7
// NOTE: See OAuth2AuthCodeRequest.possible_state, in which we can pass a state token (see build_possible_state_for_callback())
#[actix_web::get("/auth_callback")] // routing paths MUST match Config::google_redirect_uri! (actually it's GOOGLE_REDIRECT_URI in .env file)
pub async fn auth_code_callback(
    client_http_request: HttpRequest, // unfortunately, for callback from external, we only get this and body (most likely empty)
) -> impl Responder {
    let query_string = client_http_request.query_string();
    let query_params: HashMap<String, String> =
        serde_urlencoded::from_str(query_string).unwrap_or_else(|_| HashMap::new());
    let auth_code_response = OAuth2AuthCodeResponse {
        possible_error: query_params.get("error").map(|s| s.to_string()),
        possible_code: query_params.get("code").map(|s| s.to_string()),
    };
    let possible_state = query_params.get("state").map(|s| s.to_string());
    let state_token = possible_state.unwrap_or_else(|| "".to_string());

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

    match auth_code_response.possible_error {
        Some(_) => {
            // Error occurred, let's log it and return 500
            println!(
                "AuthCodeCallback: Error={}",
                auth_code_response.possible_error.unwrap()
            );
            HttpResponse::InternalServerError().finish()
        }
        None => {
            let db_connection: TDBConnection = conn_as_data.get_ref().clone();
            // First, if it is NOT an error, let's go ahead and request OAuth2Token from Google
            let auth_code = auth_code_response.possible_code.unwrap(); // should panic if code is not present!

            // since ther are no way to record the auth_code as persisted data (because we don't have the DB connection)
            // we will have to do as much work as possible on this callback-thread (which is not ideal)
            // 5. POST a request to TOKEN_URL_POST to get back access_token, refresh_token, expires_in, etc.
            let config = Config::from_env();
            let http_client      = web::Data::new( reqwest::Client::new());
            let token_request = 
                OAuth2TokenRequest {
                client_id: config.google_client_id.clone(),
                client_secret: config.google_client_secret.clone(),
                code: auth_code,
                redirect_uri: config.google_redirect_uri.clone(),
                grant_type: "authorization_code".to_string(),
            };
            let query_post_token_request: web::Query<OAuth2TokenRequest> = web::Query( &token_request);
            let token_result = request_and_store_google_oauth2(
                query_post_token_request, 
                http_client);

            match token_result {
                Ok(token_response) => {
                    // 6. GET user's email address from Google
                    let user_info_result = request_userinfo(token_response.access_token, http_client);
                    match user_info_result {
                        Ok(user_info_response) => {
                            // Now that we've got the user's email address, we can now build TokenData!
                            let token_data = TokenData::new(session_id, state_token, client_address, client_port, possible_client_email, access_token, possible_refresh_token, expires_in, expiry_time) ;

                            // save/persist it


                            // 7. Signal/notify/message/publish that we have a new session_id (new login) for any services who cares for that event...

                            // the end...
                            HTtpResponse::Ok().finish()
                        }
                        Err(_) => HttpResponse::InternalServerError().finish(),
                    }
                }
                Err(_) => HttpResponse::InternalServerError().finish(),
            }

            //HttpResponse::Ok().finish()
        }
    }
}

// HTTP POST to request Google OAuth2 token (TOKEN_URL_POST) via OAuth2TokenRequest
// this method is called from auth_code_callback() to get the token from Google
// Sample request:
//      POST /token HTTP/1.1
//      Host: oauth2.googleapis.com
//      Content-Type: application/json
//      
//      {
//          "client_id": "client_id_from_google_api_console",
//          "client_secret": "client_secret_from_google_api_console",
//          "code": "auth_code_obtained_from_auth_callback",
//          "redirect_uri": "http://localhost:8080/auth_callback",
//          "grant_type": "authorization_code"
//      }
// Note that the request is blocked until we get a response from Google OAuth2 (unlike
// auth_code_callback() which is a callback from Google OAuth2)
// Sample response:
//      {
//        "access_token": "1/fFAGRN
//        "expires_in": 3920,
async fn request_and_store_google_oauth2(
    query_post_token_request: web::Query<OAuth2TokenRequest>,
    http_client: web::Data<reqwest::Client>,
) -> impl Responder {
    let params = [
        ("code", query_post_token_request.code),
        ("client_id", query_post_token_request.client_id),
        ("client_secret",query_post_token_request.client_secret), 
        ("redirect_uri", query_post_token_request.redirect_uri),
        ("grant_type", query_post_token_request.grant_type),
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
                ,
                IpAddr::from_str("::1").unwrap(),
                0,
                None,
                oauth2_token_response.access_token, // The token that your application sends to authorize a Google API request.
                // A token that you can use to obtain a new access token.
                // Refresh tokens are valid until the user revokes access.
                // Again, this field is only present in this response if
                // you set the access_type parameter to offline in the
                // initial request to Google's authorization server
                oauth2_token_response.possible_refresh_token,
                oauth2_token_response.expires_in, // The remaining lifetime of the access token in seconds.
                expiry_time,
            );
            // Store token data in SQLite3 just in case something goes wrong...
            let conn: TDBConnection = conn_as_data.get_ref().clone();
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
    db_connection: web::Data<TDBConnection>,
    config: web::Data<Config>,
) -> impl Responder {
    let query_string = client_http_request.query_string();
    let query_params: HashMap<String, String> =
        serde_urlencoded::from_str(query_string).unwrap_or_else(|_| HashMap::new());
    let login_request = LoginRequest {
        possible_last_session_id: query_params
            .get("last_session_id")
            .map(|s| match s.to_string().parse::<u64>() {
                Ok(n64) => SessionIDType::ID(n64),
                Err(_) => SessionIDType::Undefined(None),
            }).unwrap_or_else(|| SessionIDType::Undefined(None)),
        possible_last_state_token: Some(query_params.get("last_state_token")
            .map(|s| s.to_string())
            .unwrap_or_else(|| make_state_token()) ),
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

    let state = OAuth2AuthCodeRequestState {
        session_id: login_request.possible_last_session_id,
        state_token: login_request.possible_last_state_token.clone(),
    };

    // reqwest Google to give us an AuthCode
    let auth_request = OAuth2AuthCodeRequest {
        client_id: config.google_client_id.clone(),
        redirect_uri: config.google_redirect_uri.clone(),
        response_type: "code".to_string(),
        scope: "email%20profile%20https://www.googleapis.com/auth/drive".to_string(),
        possible_access_type: Some("offline".to_string()),
        possible_state: encode_state_token(Some(state)),
        possible_include_granted_scopes: Some(true),
        possible_prompt: Some("consent%20select_account".to_string()), // space delimited
    };
    let http_client      = web::Data::new( reqwest::Client::new());
    let query_get_auth_request = web::Query::from_query(auth_request).unwrap();
    let schedule_auth = request_auth_code_trigger_callback(query_get_auth_request, http_client);

    // wait for player/client to consent
    //loop{}


    let mut login_response = LoginResponse {
        possible_login_error: Some("Unexpected (unhandled) Error".to_string()),
        possible_session_id: None,
        possible_state_token: None,
    };
    // I'd like to now block and wait for the signal that I've got a session_id...
    //loop {}

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
                        possible_login_error: None, // TODO: populate correct response
                        possible_session_id: None,
                        possible_state_token: None,
                    })
                    .unwrap();
                    HttpResponse::Ok().body(response_body)
                }
                Err(_) => HttpResponse::InternalServerError().finish(),
            }

            // TODO: signal (via kafka) of new session_id (new login) for any services who cares for that event...

    let response_body = serde_json::to_string(&login_response).unwrap();
    HttpResponse::Ok().body(response_body)
}

// Create an unique SessionToken (aka state) as mentioned in 'https://developers.google.com/identity/openid-connect/openid-connect'
//    # Python example:
//    # # Create a state token to prevent request forgery.
//    # # Store it in the session for later validation.
//    # state = hashlib.sha256(os.urandom(1024)).hexdigest()
//    # session['state'] = state
//    # BASH example:
//    echo $(head -c 32 /dev/urandom | base64 | tr -d '+/=')
fn make_state_token() -> String {
    // Generate 32 random bytes
    let mut random_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut random_bytes);

    // Encode the random bytes to base64
    let encoded = general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);

    // Remove the unwanted characters
    let sanitized_encoded = encoded.replace('+', "").replace('/', "").replace('=', "");

    println!("Random base64-encoded string: {}", sanitized_encoded);
    sanitized_encoded
}

/// Keep-alive route - Check if the client is authenticated and handle keep-alive
/// HTTP verb: GET
/// params: last_session_id
#[actix_web::get("/keepalive")]
pub async fn keepalive(
    client_http_request: HttpRequest,
    db_connection: web::Data<TDBConnection>,
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
