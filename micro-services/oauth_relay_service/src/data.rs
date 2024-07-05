use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::SystemTime};
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Clone)]
pub enum SessionIDType {
    Undefined(Option<bool>), // it's always None<_>()
    ID(u64),
    Hash(String),
}

// oftentimes, for asyn methods, it's safest to clone before moving that closure to the thread, so
// we need Clone to make sure it copies the data (not the ref)
#[derive(Serialize, Deserialize, Clone)]
pub struct TokenData {
    pub session_id: SessionIDType, // for example, for SQL-based (including SQLite, it's the unique-key index), for no-sql, HASH of access_token?

    pub client_address: std::net::IpAddr, // either IPv4 or IPv6
    pub client_port: u16,
    pub client_email: Option<String>, // optionally, client email address used to auth against

    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub expiry_time: SystemTime,
}
impl PartialEq for TokenData {
    // Equality is based only on the KEY (either IP+Port or Email)
    fn eq(&self, other: &Self) -> bool {
        let match_ip_port =
            self.client_address == other.client_address && self.client_port == other.client_port;
        match self.client_email {
            Some(ref email) => {
                email
                    == match other.client_email {
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
        client_address: std::net::IpAddr,
        client_port: u16,
        client_email: Option<String>,
        access_token: String,
        refresh_token: Option<String>,
        expires_in: i64,
        expiry_time: SystemTime,
    ) -> Self {
        Self {
            session_id,
            client_address,
            client_port,
            client_email: match client_email {
                None => None,
                Some(email) => Self::validate_email(email.as_str()),
            },
            access_token,
            refresh_token,
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

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now();
        now > self.expiry_time
    }

    pub fn get_key(&self) -> String {
        let email = match self.client_email {
            Some(ref email) => email.clone(),
            None => "".to_string(),
        };
        format!("{}:{}:{}", self.client_address, self.client_port, email)
    }
}

// see: https://developers.google.com/identity/protocols/oauth2/web-server#creatingclient

#[derive(Serialize)]
pub struct OAuth2AuthCodeRequest {
    pub client_id: String, // from the API Console Credentials page
    // redirect_uri: The value must exactly match one of the authorized redirect URIs for the
    // OAuth 2.0 client, which you configured in your client's API Console Credentials page.
    // If this value doesn't match an authorized redirect URI for the provided client_id you
    // will get a redirect_uri_mismatch error.
    // Note that the http or https scheme, case, and trailing slash ('/') must all match.
    pub redirect_uri: String, // URL to accept code= or error= response
    // response_type: Determines whether the Google OAuth 2.0 endpoint returns an authorization code.
    pub response_type: String, // Set the parameter value to `code` for web server applications.
    // scope: A space-delimited list of scopes that identify the resources that your application could
    // access on the user's behalf. These values inform the consent screen that Google displays
    // to the user.
    pub scope: String,
    // access_type: Indicates whether your application can refresh access tokens when the user is not
    // present at the browser. Set the value to 'offline' to get a refresh token and an access token the
    // first time your application exchanges an authorization code for a user.
    pub access_type: Option<String>, // valid parameter values are 'online', which is the default value, and 'offline'.
    // state: Any string value that your application uses to maintain state between your
    // authorization request and the authorization server's response. The server returns the
    // exact value that you send as a name=value pair in URL query component ('?') of the
    // 'redirect_uri' after the user consents to or denies your application's access request.
    pub state: Option<String>, // See the OpenID Connect documentation for an example of how to create and confirm a state token.
    pub include_granted_scopes: Option<bool>, // Enables applications to use incremental authorization to request access to additional scopes in context. If you set this parameter's value to true and the authorization request is granted, then the new access token will also cover any scopes to which the user previously granted the application access
    pub prompt: Option<String>, // A space-delimited, case-sensitive list of prompts to present the user. If you don't specify this parameter, the user will be prompted only the first time your app requests access. Possible values are 'none', 'consent', and 'select_account'.
}

// Sample response (to the  redirect_uri via parameter):
//   https://localhost:8080/authcode_callback?error=access_denied
//   https://localhost:8080/authcode_callback?code=4/P7q7W91a-oMsCeLvIaQm6bTrgtp7
#[derive(Deserialize)]
pub struct OAuth2AuthCodeResponse {
    // can I serialize Union/enum types so I don't have to make both elements Option<>?
    pub code: Option<String>,  // None if access_denied and/or other errors
    pub error: Option<String>, // None if code is present
}

// See: https://developers.google.com/identity/protocols/oauth2/web-server#exchange-authorization-code
//  - client_id	The client ID obtained from the API Console Credentials page.
//  - client_secret	The client secret obtained from the API Console Credentials page.
//  - code	The authorization code returned from the initial request to https://accounts.google.com/o/oauth2/v2/auth
//  - grant_type	As defined in the OAuth 2.0 specification, this field's value must be set to authorization_code.
//  - redirect_uri	One of the redirect URIs listed for your project in the API Console Credentials page for the given client_id.
#[derive(Serialize)]
pub struct OAuth2TokenRequest {
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
// see: https://developers.google.com/identity/protocols/oauth2/web-server#httprest
#[derive(Serialize, Deserialize)]
pub struct OAuth2TokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub token_type: String,
    pub scope: String,
    pub refresh_token: Option<String>,
}

#[derive(Deserialize)]
pub struct OAuth2UerInfoRequest {
    pub access_token: String,
}

#[derive(Serialize)]
pub struct OAuth2UserInfoResponse {
    pub email: String,
}
