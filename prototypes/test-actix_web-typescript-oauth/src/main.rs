use actix_files::Files;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, fs};

#[derive(Serialize, Deserialize, Debug)]
pub struct OAuth2TokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub id_token: String,
    pub scope: String,  // space separated, i.e. "https://www.googleapis.com/auth/userinfo.profile openid https://www.googleapis.com/auth/userinfo.email"
    pub token_type: String,
    pub possible_refresh_token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
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
    pub picture: String,
    // "hd": "some_domain.tld"
    pub hd: String,
}

async fn index() -> impl Responder {
    let content = fs::read_to_string("index.html").expect("Unable to read file");
    HttpResponse::Ok().content_type("text/html").body(content)
}

async fn script() -> impl Responder {
    let content = fs::read_to_string("script.js").expect("Unable to read file");
    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(content)
}

async fn auth_callback(client_http_request: HttpRequest) -> HttpResponse {
    // read the URI query parameters for either "code=xxx" or  "error=xxx"
    let query_string = client_http_request.query_string();
    let query_params: HashMap<String, String> =
        serde_urlencoded::from_str(query_string).unwrap_or_else(|_| HashMap::new());
    let possible_error = query_params.get("error").map(|s| s.to_string());
    let possible_code = query_params.get("code").map(|s| s.to_string());

    // Now that we've got the auth-code, let's exchange it for an access token
    let mut ret_body: String = "".to_string();
    match possible_code.clone() {
        Some(code) => {
            println!("Received code: '{}'", code);
            let client_id = std::env::var("GOOGLE_CLIENT_ID").expect("CLIENT_ID must be set");
            let client_secret =
                std::env::var("GOOGLE_CLIENT_SECRET").expect("CLIENT_SECRET must be set");
            let redirect_uri =
                std::env::var("GOOGLE_REDIRECT_URI").expect("REDIRECT_URI must be set");
            // panic if 

            let TOKEN_ENDPOINT_POST: &str = "https://oauth2.googleapis.com/token"; // POST
            println!("Requesting for access token at {} (POST)", TOKEN_ENDPOINT_POST);
            let client = reqwest::Client::new();
            // build JSON body
            let json_body = json!({
                "code": code,
                "client_id": client_id,
                "client_secret": client_secret,
                "redirect_uri": redirect_uri,
                "grant_type": "authorization_code"
            }).to_string();
            // NOTE: Even if your client-id is garbage (not the value from Google Console), you'll get auth-code from redirect_uri :shrug:
            //println!("Params: {}", json_body);    // only uncomment this if you're getting "invalid_client" error, it's most likely that your .env file is not set with ClientID
            let resp = client
                .post(TOKEN_ENDPOINT_POST)
                //.form(&params)
                .body(json_body)
                .send()
                .await
                .unwrap();
            // deserialize to OAuth2TokenResponse
            let body_utf8_to_str= String::from_utf8(resp.bytes().await.unwrap().to_vec()).unwrap();
            let token_response: OAuth2TokenResponse = serde_json::from_str(body_utf8_to_str.as_str()).unwrap() ;
            println!("Response from Google: {:?}", token_response);

            // let's use this access_token to get user info
            let USERINFO_ENDPOINT_GET: &str = "https://www.googleapis.com/oauth2/v1/userinfo"; // GET
            println!("Requesting for user info at {} (GET)", USERINFO_ENDPOINT_GET);
            let resp = client
                .get(USERINFO_ENDPOINT_GET)
                .header("Authorization", format!("{} {}", token_response.token_type, token_response.access_token))   
                .send()
                .await  
                .unwrap();
            let body_utf8_to_str= String::from_utf8(resp.bytes().await.unwrap().to_vec()).unwrap();
            let user_info_response: OAuth2UserInfoResponse = serde_json::from_str(body_utf8_to_str.as_str()).unwrap() ;
            println!("Response from Google: {:?}", user_info_response);

            let ret_body = 
                            format!(
                            "<html><body><h1>OAuth Callback</h1><p>Code: {:?}</p><p>Error: {:?}</p><p>Token Response: {:?}<p>UserInfo: {:?}</p><></body></html>",
                            possible_code, possible_error, token_response , user_info_response
                            );
                        );
            println!("\n{}\n", ret_body);
        }
        None => {
            println!("No code received");
        }
    }
    HttpResponse::Ok().content_type("text/html").body(ret_body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let redirect_uri = std::env::var("GOOGLE_REDIRECT_URI").expect("REDIRECT_URI must be set");
    // parse redirect_uri (i.e. "http://localhost:8080/auth_callback") and extract port
    let port_from_redirect = redirect_uri.split(":").collect::<Vec<&str>>()[2]
        .split("/")
        .collect::<Vec<&str>>()[0];
    let port = port_from_redirect
        .parse::<u16>()
        .expect("Port must be a number");
    let local_service = format!("127.0.0.1:{port}", port = port);

    println!("#######################################################");
    println!("Open browser and navigate to {}", local_service);
    println!("Press Ctrl+C to stop this server");
    println!("#######################################################");

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/script.js", web::get().to(script))
            .route("/auth_callback", web::get().to(auth_callback)) // make sure this matches the redirect_uri in .env
    })
    .bind(local_service)?
    .run()
    .await
}
