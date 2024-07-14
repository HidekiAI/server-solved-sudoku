use crate::{
    config::Config,
    data::*,
    messenger::{self, TMQConnectionLock},
    storage::{self, TDBConnectionLock},
    web::web_consts::*,
};
use actix_web::{
    web, HttpRequest, HttpResponse,
};
use anyhow::Result as AnyResult;
use base64::{engine::general_purpose, Engine};
use rand::RngCore;
use serde_urlencoded;
use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};
use tokio::{time::sleep};

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
    http_client: &web::Data<reqwest::Client>,
) -> Result<reqwest::Response, reqwest::Error> {
    let response = http_client
        .get(AUTHORIZATION_ENDPOINT_GET)
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
            &format!(
                "state={}",
                match query_get_auth_request.possible_state.clone() {
                    Some(state) => state,
                    None => "".to_string(),
                }
            ),
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
) -> HttpResponse {
    let query_string = client_http_request.query_string();
    let query_params: HashMap<String, String> =
        serde_urlencoded::from_str(query_string).unwrap_or_else(|_| HashMap::new());
    let auth_code_response = OAuth2AuthCodeResponse {
        possible_error: query_params.get("error").map(|s| s.to_string()),
        possible_code: query_params.get("code").map(|s| s.to_string()),
    };
    let possible_state = decode_state_token(query_params.get("state").map(|s| s.to_string()));

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
            let config = Config::from_env("./build/.env");
            let db_connection = storage::open_db_connection_from_config(config.clone()).await;
            let mq_connection = messenger::open_mq_connection_from_config(config.clone()).await;

            // First, if it is NOT an error, let's go ahead and request OAuth2Token from Google
            let auth_code = auth_code_response.possible_code.unwrap(); // should panic if code is not present!

            let state_token = match possible_state {
                Some(s) => s.state_token,
                None => make_state_token(), // we should panic, but if we've come this far, we should take it
            };

            // since ther are no way to record the auth_code as persisted data (because we don't have the DB connection)
            // we will have to do as much work as possible on this callback-thread (which is not ideal)
            // 5. POST a request to TOKEN_URL_POST to get back access_token, refresh_token, expires_in, etc.
            let http_client = web::Data::new(reqwest::Client::new());
            let token_request = OAuth2TokenRequest {
                state_token: state_token.clone(),
                client_id: config.google_client_id.clone(),
                client_secret: config.google_client_secret.clone(),
                code: auth_code,
                redirect_uri: config.google_redirect_uri.clone(),
                grant_type: "authorization_code".to_string(),
            };
            let query_post_token_request: web::Query<OAuth2TokenRequest> =
                web::Query(token_request);
            let possible_token_response =
                request_access_token(&query_post_token_request, &http_client).await;

            match possible_token_response {
                Ok(oauth_token_response) => {
                    //let token_data: TokenData = serde_json::from_str( std::str::from_utf8(token_response_json_body.body().into().as_bytes) .unwrap(),) .unwrap();
                    // 6. GET user's email address from Google
                    let user_info_result =
                        request_userinfo(&http_client, oauth_token_response.clone()).await;
                    match user_info_result {
                        Ok(user_info_response) => {
                            let next_expected_time = SystemTime::now()
                                .checked_add(Duration::from_secs(
                                    oauth_token_response.expires_in.clone() as u64,
                                )) // Google OAuth2 token expires_in is in seconds
                                .unwrap();
                            // Now that we've got the user's email address, we can now build TokenData!
                            let token_data = TokenData::new(
                                SessionIDType::make_hash(state_token.clone().as_str()),
                                state_token.clone(),
                                client_ip,
                                client_port,
                                Some(user_info_response.email),
                                oauth_token_response.access_token,
                                oauth_token_response.possible_refresh_token,
                                oauth_token_response.expires_in,
                                next_expected_time,
                            );

                            // save/persist it
                            storage::upsert_token_data(&config, &db_connection, &token_data);

                            // 7. Signal/notify/message/publish that we have a new session_id (new login) for any services who cares for that event...
                            messenger::post_new_login(&config, &mq_connection, &token_data);

                            // the end...
                            HttpResponse::Ok().finish()
                        }
                        Err(_) => HttpResponse::InternalServerError().finish(),
                    }
                }
                Err(e) => HttpResponse::InternalServerError().finish(),
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
//  {
//      "access_token" : "ya29.a0AXooCgs2qcHYleg5gD_Qrm...BFQ3NfyrB61-171",
//      "expires_in" : 3599,
//      "id_token" : "eyJhbGciOiJSUzI1NiIsIm...tpZCI6Ijg3YmJlMDmQYA",
//      "scope" : "https://www.googleapis.com/auth/userinfo.profile openid https://www.googleapis.com/auth/userinfo.email",
//      "token_type" : "Bearer"
//   }
async fn request_access_token(
    query_post_token_request: &web::Query<OAuth2TokenRequest>,
    http_client: &web::Data<reqwest::Client>,
) -> AnyResult<OAuth2TokenResponse> {
    let params = [
        ("code", query_post_token_request.code.clone()),
        ("client_id", query_post_token_request.client_id.clone()),
        (
            "client_secret",
            query_post_token_request.client_secret.clone(),
        ),
        (
            "redirect_uri",
            query_post_token_request.redirect_uri.clone(),
        ),
        ("grant_type", query_post_token_request.grant_type.clone()),
    ];

    let response = http_client
        .post(TOKEN_ENDPOINT_POST)
        .form(&params)
        .send()
        .await;
    match response {
        Ok(resp) => {
            // deserialize the response from Google OAuth2
            let oauth2_token_response: OAuth2TokenResponse = resp.json().await.unwrap();
            Ok(oauth2_token_response)
        }
        Err(_) => anyhow::bail!("Failed to get token from Google OAuth2"),
    }
}

// this method is indirectly called from auth_code_callback() (chained call when qurest_and_store_google_oauth2 succeeds)
// get the user's email address from Google
async fn request_userinfo(
    http_client: &web::Data<reqwest::Client>,
    token_response: OAuth2TokenResponse,
) -> AnyResult<OAuth2UserInfoResponse> {
    // once we got OK/200 from Google OAuth2, get e-mail address (make sure Google API was setup with email priv enabled)
    // $curl -X GET "https://www.googleapis.com/oauth2/v1/userinfo?alt=json" -H"Authorization: Bearer accessTokenHere"
    // JavaScript example:
    //    var xhr = new XMLHttpRequest();
    //    xhr.open('POST', 'https://vision.googleapis.com/v1/images:annotate');
    //    xhr.setRequestHeader('Authorization', 'Bearer ' + token);
    //    xhr.setRequestHeader('Content-Type', 'application/json; charset=utf-8');
    //    xhr.setRequestHeader('x-content-type-options', 'nosniff'); // Add x-content-type-options header
    //    xhr.onload = function () {...}
    let req = OAuth2UerInfoRequest {
        access_token: token_response.access_token.clone(),
        // i.e. "Autorization": "Bearer <access_token_here>"
        header_authorization: format!(
            "{} {}", // note that because this is header, we don't need to use '%20' for space
            token_response.token_type.clone(),
            token_response.access_token.clone()
        ),
    };
    let user_info_http_response = http_client
        .get(USERINFO_ENDPOINT_GET)
        .header("Authorization", req.header_authorization)
        .send()
        .await
        .unwrap();

    // wrap LoginResponse in a Result (Body)
    //let possible_user_info_response = httpresponse_body_to_json::<OAuth2UserInfoResponse>(user_info_response).await;
    let possible_user_info_response = user_info_http_response
        .json::<OAuth2UserInfoResponse>()
        .await;
    match possible_user_info_response {
        Ok(user_info) => Ok(user_info),
        Err(_) => anyhow::bail!("Failed to get user info from Google OAuth2"),
    }
}

/// Login route - either authenticate or re-authenticate the client against Google OAuth2
/// HTTP verb: GET
/// params: last_session_id (optional)
/// see: https://developers.google.com/static/identity/protocols/oauth2/images/flows/authorization-code.png
#[actix_web::get("/login")]
pub async fn login(
    client_http_request: HttpRequest,
    db_connection: web::Data<TDBConnectionLock>,
    mq_connection: web::Data<TMQConnectionLock>,
    config: web::Data<Config>,
) -> HttpResponse {
    //let db_connection = storage::open_db_connection_from_config(config.clone()).await;
    //let mq_connection = messenger::open_mq_connection_from_config(config.clone()).await;
    let query_string = client_http_request.query_string();
    let query_params: HashMap<String, String> =
        serde_urlencoded::from_str(query_string).unwrap_or_else(|_| HashMap::new());
    let login_request = LoginRequest {
        possible_last_session_id: query_params
            .get("last_session_id")
            .map(|s| match s.to_string().parse::<u64>() {
                Ok(n64) => SessionIDType::ID(n64),
                Err(_) => SessionIDType::Undefined(None),
            })
            .unwrap_or_else(|| SessionIDType::Undefined(None)),
        possible_last_state_token: Some(
            query_params
                .get("last_state_token")
                .map(|s| s.to_string())
                .unwrap_or_else(|| make_state_token()),
        ),
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
        login_client_ip: client_ip,
        login_client_port: client_port,

        session_id: login_request.possible_last_session_id,
        state_token: match login_request.possible_last_state_token {
            Some(s) => s.clone(),
            None => make_state_token(),
        },

        db_type: storage::get_db_type(config.as_ref()),
        possible_db_address: storage::get_db_address(config.as_ref()),
        possible_db_port: storage::get_db_port(config.as_ref()),
        possible_db_path: storage::get_db_path(config.as_ref()),

        mq_type: messenger::get_mq_type(config.as_ref()),
        possible_mq_address: messenger::get_mq_address(config.as_ref()),
        possible_mq_port: messenger::get_mq_port(config.as_ref()),
    };

    // reqwest Google to give us an AuthCode
    let auth_request = OAuth2AuthCodeRequest {
        client_id: config.google_client_id.clone(),
        redirect_uri: config.google_redirect_uri.clone(),
        response_type: "code".to_string(), // "code" for Authorization Code flow, valid: "code", "token", "id_token", "code token", "code id_token", "token id_token", "code token id_token"
        scope: "email%20profile%20https://www.googleapis.com/auth/drive".to_string(),
        possible_access_type: Some("offline".to_string()),
        possible_state: encode_state_token(Some(state.clone())),
        possible_include_granted_scopes: Some(true),
        possible_prompt: Some("consent%20select_account".to_string()), // space delimited
    };
    let auth_request_as_json = serde_json::to_string(&auth_request).unwrap();
    let http_client = web::Data::new(reqwest::Client::new());
    let query_get_auth_request = web::Query::from_query(auth_request_as_json.as_str()).unwrap();
    let schedule_auth = request_auth_code_trigger_callback(query_get_auth_request, &http_client);

    // wait for player/client to consent
    //loop{}

    let login_response = LoginResponse {
        possible_login_error: Some("Unexpected (unhandled) Error".to_string()),
        possible_session_id: None,
        possible_state_token: None,
    };
    // Block and wait for the signal that I've got a session_id...
    let mut possible_token_response: Option<TokenData> = None;
    let start_time = SystemTime::now();
    let timeout = TIMEOUT_FOR_AUTH_CODE_CALLBACK;
    loop {
        // check if we have a session_id from messenger
        possible_token_response = messenger::get_token(
            &config,
            &mq_connection,
            &state.state_token.as_str(),
            |lhs_state_token: &str, token_data: TokenData| {
                lhs_state_token == token_data.state_token.as_str()
            },
        );

        // yield
        sleep(Duration::from_secs(1));
        if start_time.elapsed().unwrap() > timeout {
            break;
        }
    }

    // Now, query for user's email client_address
    match possible_token_response {
        Some(resp) => {
            // deserialize the response from Google OAuth2
            // wrap LoginResponse in a Result (Body)
            let response_body = serde_json::to_string(&LoginResponse {
                possible_login_error: None,
                possible_session_id: resp.session_id(),
                possible_state_token: Some(resp.state_token),
            })
            .unwrap();
            HttpResponse::Ok().body(response_body)
        }
        None => HttpResponse::InternalServerError().finish(),
    };

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
