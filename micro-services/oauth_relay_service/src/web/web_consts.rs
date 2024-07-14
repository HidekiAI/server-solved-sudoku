use crate::{
    config::Config, 
    data::*, 
    messenger::{self, open_mq_connection_from_config}, 
    storage::{TDBConnection}
};    
use anyhow::Result as AnyResult;
use base64::{engine::general_purpose, Engine};
use core::net;
use futures::task::Spawn;
use oauth2::http::response;
use rand::RngCore;
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

pub const TOKEN_REFRESH_INTERVAL: Duration = Duration::from_secs(3600);
pub const TOKEN_REFRESH_INTERVAL_MARGIN: Duration = Duration::from_secs(30);

pub const TIMEOUT_FOR_AUTH_CODE_CALLBACK: Duration = Duration::from_secs(60);

// NOTE: To make it less error-prone, the real way to do this is to grab the (latest)
//      JSON document from https://accounts.google.com/.well-known/openid-configuration
//      and extract currently defined endpoints from there...
// See: https://developers.google.com/identity/openid-connect/openid-connect#discovery
pub const AUTHORIZATION_ENDPOINT_GET: &str = "https://accounts.google.com/o/oauth2/v2/auth"; // GET
pub const TOKEN_ENDPOINT_POST: &str = "https://oauth2.googleapis.com/token"; // POST
pub const USERINFO_ENDPOINT_GET: &str = "https://openidconnect.googleapis.com/v1/userinfo"; // GET

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
//     via request_access_token()
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
