use super::data_postgres::{AuthRequest, SharedTokens, TokenData, TokenResponse};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    prelude::*,
    sync::Mutex,
    time::{timeout, Duration},
};
use tokio_postgres::{Client as PgClient, Error};

async fn get_token(client: &PgClient, client_id: &str) -> Result<Option<TokenData>, Error> {
    let row = client.query_opt(
        "SELECT access_token, refresh_token, expires_in, expiry_time FROM tokens WHERE client_id = $1",
        &[&client_id],
    ).await?;

    if let Some(row) = row {
        let token_data = TokenData {
            access_token: row.get(0),
            refresh_token: row.get(1),
            expires_in: row.get(2),
            expiry_time: UNIX_EPOCH + Duration::from_secs(row.get::<_, i64>(3) as u64),
        };
        Ok(Some(token_data))
    } else {
        Ok(None)
    }
}

pub async fn start_tcp_server_atix(tokens: SharedTokens, pg_client: PgClient) {
    let listener = TcpListener::bind("0.0.0.0:8081").await.unwrap();
    println!("TCP server running on port 8081");

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        let tokens = tokens.clone();
        //let pg_client = pg_client.clone();

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            loop {
                match timeout(Duration::from_secs(30), socket.read(&mut buf)).await {
                    Ok(Ok(n)) if n == 0 => {
                        // Connection closed
                        println!("Connection closed");
                        break;
                    }
                    Ok(Ok(n)) => {
                        // Handle received data
                        println!("Received {} bytes: {:?}", n, &buf[..n]);
                        // Example: Parse the client ID and handle token data
                        let client_id = "example_client_id"; // This should be extracted from the received data

                        // Example of inserting or updating token data
                        let token_data = TokenData {
                            access_token: "example_access_token".to_string(),
                            refresh_token: Some("example_refresh_token".to_string()),
                            expires_in: 3600,
                            expiry_time: SystemTime::now() + Duration::new(3600, 0),
                        };

                        let mut tokens = tokens.lock().await;
                        tokens.insert(client_id.to_string(), token_data);
                    }
                    Ok(Err(e)) => {
                        println!("Failed to read from socket; err = {:?}", e);
                        break;
                    }
                    Err(_) => {
                        println!("Timeout: closing connection");
                        break;
                    }
                }
            }
        });
    }
}

async fn handle_client(mut socket: TcpStream, tokens: SharedTokens, pg_client: PgClient) {
    let mut buf = [0; 1024];

    loop {
        match timeout(Duration::from_secs(30), socket.read(&mut buf)).await {
            Ok(Ok(n)) if n == 0 => {
                // Connection closed
                println!("Connection closed");
                break;
            }
            Ok(Ok(n)) => {
                // Handle received data
                println!("Received {} bytes: {:?}", n, &buf[..n]);
                // Example: Parse the client ID and handle token data
                let client_id = "example_client_id"; // Extract from received data

                // Example of inserting or updating token data
                let token_data = TokenData {
                    access_token: "example_access_token".to_string(),
                    refresh_token: Some("example_refresh_token".to_string()),
                    expires_in: 3600,
                    expiry_time: SystemTime::now() + Duration::new(3600, 0),
                };

                let mut tokens = tokens.lock().await;
                tokens.insert(client_id.to_string(), token_data);
            }
            Ok(Err(e)) => {
                println!("Failed to read from socket; err = {:?}", e);
                break;
            }
            Err(_) => {
                println!("Timeout: closing connection");
                break;
            }
        }
    }
}

pub async fn start_tcp_server_tokio(tokens: SharedTokens, pg_client: PgClient) {
    let listener = TcpListener::bind("0.0.0.0:8081").await.unwrap();
    println!("TCP server running on port 8081");

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let tokens = tokens.clone();
        let pg_client = pg_client.clone();

        tokio::spawn(async move {
            handle_client(socket, tokens, pg_client).await;
        });
    }
}

//async fn start_tcp_server_tokio(tokens: SharedTokens, pg_client: PgClient) {
//    let listener = TcpListener::bind("0.0.0.0:8081").await.unwrap();
//    println!("TCP server running on port 8081");
//
//    loop {
//        let (mut socket, _) = listener.accept().await.unwrap();
//        let tokens = tokens.clone();
//        //let pg_client = pg_client.clone();
//
//        tokio::spawn(async move {
//            let mut buf = [0; 1024];
//
//            loop {
//                match timeout(Duration::from_secs(30), socket.read(&mut buf)).await {
//                    Ok(Ok(n)) if n == 0 => {
//                        // Connection closed
//                        println!("Connection closed");
//                        break;
//                    }
//                    Ok(Ok(n)) => {
//                        // Handle received data
//                        println!("Received {} bytes: {:?}", n, &buf[..n]);
//                        // Example: Parse the client ID and handle token data
//                        let client_id = "example_client_id"; // Extract from received data
//
//                        // Example of inserting or updating token data
//                        let token_data = TokenData {
//                            access_token: "example_access_token".to_string(),
//                            refresh_token: Some("example_refresh_token".to_string()),
//                            expires_in: 3600,
//                            expiry_time: SystemTime::now() + Duration::new(3600, 0),
//                        };
//
//                        let mut tokens = tokens.lock().await;
//                        tokens.insert(client_id.to_string(), token_data);
//                    }
//                    Ok(Err(e)) => {
//                        println!("Failed to read from socket; err = {:?}", e);
//                        break;
//                    }
//                    Err(_) => {
//                        println!("Timeout: closing connection");
//                        break;
//                    }
//                }
//            }
//        });
//    }
//}
//
