use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};
use std::{
    time::{SystemTime, UNIX_EPOCH},
};

use crate::config::HostType;

//pub async fn httpresponse_body_to_json(http_response: reqwest::Response) -> AnyResult<TokenData> {
//    let vec_u8 = http_response.bytes().await.unwrap().to_vec();
//    let token_data = serde_json::from_str(
//        std::str::from_utf8(&vec_u8)
//            .map_err(|e| anyhow::Error::new(e))
//            .unwrap(),
//    )
//    .map_err(|e| anyhow::Error::new(e));
//    match token_data {
//        Ok(token_data) => Ok(token_data),
//        Err(e) => Err(e),
//    }
//}

//// NOTE: I cannot do `http_response: &reqwest::Response` because it does not implement Copy trait
//pub async fn httpresponse_body_to_json<'a, T: serde::Deserialize<'a> >(http_response: reqwest::Response) -> AnyResult<T> {
//    let vec_u8 = http_response.bytes().await.unwrap().to_vec();
//    let token_data: Result<T, anyhow::Error> = serde_json::from_slice(&vec_u8)
//        .map_err(|e| anyhow::Error::new(e));
//    match token_data {
//        Ok(token_data) => Ok(token_data),
//        Err(e) => Err(e),
//    }
//}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SessionIDType {
    Undefined(Option<bool>), // it's always None<_>()
    ID(u64),
    Hash(u64),
}
impl SessionIDType {
    pub fn make_hash(session_token: &str) -> SessionIDType {
        let hash = crc64::crc64(0, session_token.as_bytes());
        SessionIDType::Hash(hash)
    }
}

// oftentimes, for asyn methods, it's safest to clone before moving that closure to the thread, so
// we need Clone to make sure it copies the data (not the ref)
#[derive(Serialize, Deserialize, Clone)]
pub struct TokenData {
    session_id: SessionIDType, // for example, for SQL-based (including SQLite, it's the unique-key index), for no-sql, HASH of access_token?
    pub state_token: String,   // used to prevent CSRF attacks

    pub client_address: HostType, // either IPv4 or IPv6
    pub client_port: u16,
    pub possible_client_email: Option<String>, // optionally, client email address used to auth against

    pub access_token: String,
    pub possible_refresh_token: Option<String>,
    pub expires_in: i64,
    expiry_time: SystemTime,
}
// override to_string() so that we won't print access_token and other sensitive data in logs
impl std::fmt::Display for TokenData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "TokenData {{ session_id: {:?}, state_token: {}, client_address: {}, client_port: {}, possible_client_email: {:?}, access_token: <hidden>, possible_refresh_token: <hidden>, expires_in: {}, expiry_time: {:?} }}",
            self.session_id,
            self.state_token,
            self.client_address,
            self.client_port,
            self.possible_client_email,
            self.expires_in,
            self.expiry_time
        )
    }
}

impl PartialEq for TokenData {
    // Equality is based only on the KEY (either IP+Port or Email)
    fn eq(&self, other: &Self) -> bool {
        let match_ip_port =
            self.client_address == other.client_address && self.client_port == other.client_port;
        match self.possible_client_email {
            Some(ref email) => {
                email
                    == match other.possible_client_email {
                        Some(ref other_email) => other_email,
                        None => return false,
                    }
                    && match_ip_port
            }
            None => match_ip_port,
        }
    }
}
impl TokenData {
    pub fn new(
        session_id: SessionIDType,
        state_token: String,
        client_address: HostType,
        client_port: u16,
        possible_client_email: Option<String>,
        access_token: String,
        possible_refresh_token: Option<String>,
        expires_in: i64,
        expiry_time: SystemTime,
    ) -> Self {
        Self {
            session_id,
            state_token,
            client_address,
            client_port,
            possible_client_email: match possible_client_email {
                None => None,
                Some(email) => Self::validate_email(email.as_str()),
            },
            access_token,
            possible_refresh_token,
            expires_in,
            expiry_time,
        }
    }

    fn validate_email(email: &str) -> Option<String> {
        match email.to_string().to_lowercase().trim() {
            "" => None,
            se => {
                // TODO: Verify that it has one and only one "@" and  "." (fubar@mail.foo.bar is not allowed)
                Some(se.to_string())
            }
        }
    }

    pub fn expiry_time(&self) -> SystemTime {
        self.expiry_time
    }
    pub fn expiry_time_as_sec_from_epoch(&self) -> u64 {
        // NOTE: If this panics, it's because the expiry_time is BEFORE UNIX_EPOCH, which is impossible!
        self.expiry_time
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
    // can be negative, to indicate the past, 0 to be Now(), and +t to be in the future
    pub fn expiry_time_as_sec_from_now(&self) -> i64 {
        let expiry_time_since_epoch = self
            .expiry_time
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let now_in_seconds_since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        expiry_time_since_epoch - now_in_seconds_since_epoch
    }

    // set absolute time, regardless of the time being in the past
    pub fn set_expiry_time(&mut self, expiry_time: SystemTime) {
        self.expiry_time = expiry_time;
        match self.expiry_time_as_sec_from_now() {
            d if d < 0 => {
                // warn that the expiry time is in the past
                println!(
                    "TokenData::set_expiry_time(): Expiry time to be set is in the past: {:?}",
                    expiry_time
                );
            }
            _ => {}
        }
    }
    // duration can be negative, to indicate the past, 0 to be Now(), and +t to be in the future
    pub fn set_expiry_time_from_now(&mut self, duration_in_sec: i64) {
        match duration_in_sec {
            0 => {
                // set the expiry time to now
                self.expiry_time = SystemTime::now();
            }
            d if d < 0 => {
                // set the expiry time to 'now - abs(duration)' (rewind, go back in time)
                self.expiry_time = SystemTime::now()
                    .checked_sub(std::time::Duration::from_secs(duration_in_sec.abs() as u64))
                    .unwrap();
            }
            _ => {
                // set the expiry time to now + duration
                self.expiry_time =
                    SystemTime::now() + std::time::Duration::from_secs(duration_in_sec as u64);
            }
        }
        // set the expiry time to now + aboslute_value(duration)
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now();
        now > self.expiry_time
    }

    pub fn get_key(&self) -> String {
        let email = match self.possible_client_email {
            Some(ref email) => email.clone(),
            None => "".to_string(),
        };
        format!("{}:{}:{}", self.client_address, self.client_port, email)
    }

    pub fn session_id(&self) -> Option<u64> {
        match self.session_id {
            SessionIDType::ID(id) => Some(id),
            SessionIDType::Hash(hash) => Some(hash),
            _ => None,
        }
    }
}

// see: https://developers.google.com/identity/protocols/oauth2/web-server#creatingclient
// HTTP GET request to AUTH_URL_GET:
//   https://accounts.google.com/o/oauth2/v2/auth?response_type=code&client_id=my_client_id&redirect_uri=my_redirect_uri&scope=scope&state=my_state
//   &access_type=my_access_type&include_granted_scopes=my_include_granted_scopes
//   &prompt=select_account%20consent
#[derive(Serialize, Deserialize, Clone)]
pub struct OAuth2AuthCodeRequest {
    pub client_id: String, // from the API Console Credentials page

    // redirect_uri: The value must exactly match one of the authorized redirect URIs for the
    // OAuth 2.0 client, which you configured in your client's API Console Credentials page.
    // If this value doesn't match an authorized redirect URI for the provided client_id you
    // will get a redirect_uri_mismatch error.
    // Note that the http or https scheme, case, and trailing slash ('/') must all match.
    pub redirect_uri: String, // URL to accept code= or error= response

    // response_type: Determines whether the Google OAuth 2.0 endpoint returns an authorization code.
    // See "response_types_supported" in https://accounts.google.com/.well-known/openid-configuration
    pub response_type: String, // Set the parameter value to `code` for web server applications.

    // scope: A space-delimited list of scopes that identify the resources that your application could
    // access on the user's behalf. These values inform the consent screen that Google displays
    // to the user.
    // See: https://developers.google.com/identity/openid-connect/openid-connect#obtaininguserprofileinformation
    //      for access to UserInfo endpoint (HTTPS GET).
    //      Example: "openid%20profile%20email" ("openid profile email")
    // For supported types, see the "scopes_supported" array in https://accounts.google.com/.well-known/openid-configuration
    // See also: https://developers.google.com/identity/protocols/oauth2/scopes#openid_connect (search for "Google Sign-in")
    pub scope: String,

    // access_type: Indicates whether your application can refresh access tokens when the user is not
    // present at the browser. Set the value to 'offline' to get a refresh token and an access token the
    // first time your application exchanges an authorization code for a user.
    pub possible_access_type: Option<String>, // valid parameter values are 'online', which is the default value, and 'offline'.

    // state: Any string value that your application uses to maintain state between your
    // authorization request and the authorization server's response. The server returns the
    // exact value that you send as a name=value pair in URL query component ('?') of the
    // 'redirect_uri' after the user consents to or denies your application's access request.
    // See the OpenID Connect documentation for an example of how to create and confirm a state token.
    pub possible_state: Option<String>, // See encode_state_token() and decode_state_token()

    pub possible_include_granted_scopes: Option<bool>, // Enables applications to use incremental authorization to request access to additional scopes in context. If you set this parameter's value to true and the authorization request is granted, then the new access token will also cover any scopes to which the user previously granted the application access
    pub possible_prompt: Option<String>, // A space-delimited, case-sensitive list of prompts to present the user. If you don't specify this parameter, the user will be prompted only the first time your app requests access. Possible values are 'none', 'consent', and 'select_account'.
}

// state: Any string value that your application uses to maintain state between your
// authorization request and the authorization server's response. The server returns the
// exact value that you send as a name=value pair in URL query component ('?') of the
// 'redirect_uri' after the user consents to or denies your application's access request.
// See the OpenID Connect documentation for an example of how to create and confirm a state token.
#[derive(Serialize, Deserialize, Clone)]
pub struct OAuth2AuthCodeRequestState {
    pub login_client_ip: HostType,
    pub login_client_port: u16,

    pub session_id: SessionIDType,
    pub state_token: String,

    pub db_type: String,                               // SQLite or PostgresSQL
    pub possible_db_address: Option<HostType>, // for PostgresSQL
    pub possible_db_port: Option<u16>,                 // for PostgresSQL
    pub possible_db_path: Option<String>,              // for SQLite

    pub mq_type: String,                               // None, Kafka, etc
    pub possible_mq_address: Option<HostType>, // for PostgresSQL
    pub possible_mq_port: Option<u16>,                 // for PostgresSQL
}
pub fn encode_state_token(possible_state: Option<OAuth2AuthCodeRequestState>) -> Option<String> {
    match possible_state {
        Some(state) =>
        // serialize to JSON and encode it to base64
        {
            // const CUSTOM_ENGINE: engine::GeneralPurpose =
            //     engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);
            // let b64_url = CUSTOM_ENGINE.encode(b"hello internet~");
            let str_state = serde_json::to_string(&state).unwrap(); // panic on failure
            let encoded: String = general_purpose::URL_SAFE_NO_PAD.encode(str_state.as_bytes());
            Some(encoded)
        }
        None => None,
    }
}
pub fn decode_state_token(state_token: Option<String>) -> Option<OAuth2AuthCodeRequestState> {
    match state_token {
        Some(state) => {
            // decode from base64 and deserialize from JSON to struct
            let decoded_bytes = general_purpose::URL_SAFE_NO_PAD.decode(state).unwrap(); // panic on failure
            let decoded = String::from_utf8(decoded_bytes).unwrap(); // panic on failure
            let possible_state = serde_json::from_str(&decoded);
            match possible_state {
                Ok(state) => Some(state),
                Err(_) => None,
            }
        }
        None => None,
    }
}

// Sample response (to the  redirect_uri via parameter):
//   https://localhost:8080/authcode_callback?error=access_denied
//   https://localhost:8080/authcode_callback?code=4/P7q7W91a-oMsCeLvIaQm6bTrgtp7
#[derive(Deserialize, Clone)]
pub struct OAuth2AuthCodeResponse {
    // can I serialize Union/enum types so I don't have to make both elements Option<>?
    pub possible_code: Option<String>, // None if access_denied and/or other errors
    pub possible_error: Option<String>, // None if code is present
}

// See: https://developers.google.com/identity/protocols/oauth2/web-server#exchange-authorization-code
// HTTP POST request to TOKEN_URL_POST
//  - client_id	The client ID obtained from the API Console Credentials page.
//  - client_secret	The client secret obtained from the API Console Credentials page.
//  - code	The authorization code returned from the initial request to https://accounts.google.com/o/oauth2/v2/auth
//  - grant_type	As defined in the OAuth 2.0 specification, this field's value must be set to authorization_code.
//  - redirect_uri	One of the redirect URIs listed for your project in the API Console Credentials page for the given client_id.
#[derive(Serialize, Clone)]
pub struct OAuth2TokenRequest {
    pub state_token: String,
    pub client_id: String,
    pub client_secret: String,
    pub code: String,
    pub redirect_uri: String,
    pub grant_type: String,
}

// Sample response:
//  {
//      "access_token": "1/fFAGRNJru1FTz70BzhT3Zg",
//      "expires_in": 3920,
//      "token_type": "Bearer",
//      "scope": "https://www.googleapis.com/auth/drive.metadata.readonly",
//      "refresh_token": "1//xEoDL4iW3cxlI7yDbSRFYNG01kVKM2C-259HOF2aQbI"
//  }
//  Actual response (edited/shortened) from Google:
//  {
//      "access_token" : "ya29.a0AXooCgs2qcHYleg5gD_Qrm...BFQ3NfyrB61-171",
//      "expires_in" : 3599,
//      "id_token" : "eyJhbGciOiJSUzI1NiIsIm...tpZCI6Ijg3YmJlMDmQYA",
//      "scope" : "https://www.googleapis.com/auth/userinfo.profile openid https://www.googleapis.com/auth/userinfo.email",
//      "token_type" : "Bearer"
//   }
// see: https://developers.google.com/identity/protocols/oauth2/web-server#httprest
#[derive(Serialize, Deserialize, Clone)]
pub struct OAuth2TokenResponse {
    pub access_token: String,
    pub expires_in: i64, // I believe Google uses seconds (i.e. 3599 (6 minutes))
    pub id_token: String,
    pub scope: String, // space separated, i.e. "https://www.googleapis.com/auth/userinfo.profile openid https://www.googleapis.com/auth/userinfo.email"
    pub token_type: String,
    pub possible_refresh_token: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct OAuth2UerInfoRequest {
    pub access_token: String,
    pub header_authorization: String, // "Bearer <access_token>"
}

//  Response from Google:
//  {
//    "id": "113980003478662",
//    "email": "hidekiai@some_domain.tld",
//    "verified_email": true,
//    "name": "Hideki A. Ikeda",
//    "given_name": "Hideki A.",
//    "family_name": "Ikeda",
//    "picture": "https://lh3.googleusercontent.com/some_link_to_image.jpg",
//    "hd": "some_domain.tld"
//  }
//  BODY:
//<html><body><h1>OAuth Callback</h1>
//<p>Code: Some("4/0ATx3LY7MX-iBJmPPujTw...amdJEO1mxg")</p>
//<p>Error: None</p>
//<p>OAuth2TokenResponse {
//      access_token: "ya29.a0AXooCgvskAow...Rfl_6_AybHG2Gmy9Mf0171",
//      expires_in: 3599,
//      id_token: "eyJhbGciOiJ...SUzI1NiIVsFwQ",
//      scope: "openid https://www.googleapis.com/auth/userinfo.email https://www.googleapis.com/auth/userinfo.profile",
//      token_type: "Bearer",
//      possible_refresh_token: None }</p>
// </body></html>
#[derive(Serialize, Deserialize, Clone)]
pub struct OAuth2UserInfoResponse {
    // "id": "113983517891773478662",
    pub id: String,
    // "email": "hidekiai@some_domain.tld",
    pub email: String,
    // "verified_email": true,
    pub verified_email: bool,
    // "name": "Hideki A. Ikeda",
    pub name: String,
    // "given_name": "Hideki A.",
    pub given_name: String,
    // "family_name": "Ikeda",
    pub family_name: String,
    // "picture": "https://lh3.googleusercontent.com/some-link-to-image.jpg",
    pub picture: Option<String>,
    // "hd": "some_domain.tld"
    pub hd: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct LoginRequest {
    pub possible_last_session_id: SessionIDType,
    pub possible_last_state_token: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct LoginResponse {
    // Either session info (for next login request/recovery/keeplive) or error message
    pub possible_session_id: Option<u64>,
    pub possible_state_token: Option<String>,

    pub possible_login_error: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct KeepaliveRequest {
    pub last_session_id: String,
}
#[derive(Serialize, Clone)]
pub struct KeepaliveResponse {
    pub next_expected_time: u64, // absolute time, EPOCH based
    // time-to-live in seconds (problem with relative deltaT is that if round-trip takes long time,
    // few seconds may have passed already...) probably better to just calcuate
    // TTL = next_expected_time - current_time on client side...
    pub ttl: u64,
    pub status: String,
    pub message: String,
}
