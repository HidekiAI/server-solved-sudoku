use super::data_redis::{AuthRequest, SharedTokens, TokenData, TokenResponse};
use redis::AsyncCommands;
use redis::Client as RedisClient;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    prelude::*,
    sync::Mutex,
    time::{timeout, Duration},
};

async fn get_token(redis_client: &RedisClient, key: &str) -> redis::RedisResult<Option<TokenData>> {
    let mut conn = redis_client.get_async_connection().await?;
    let data: Option<String> = conn.get(key).await?;
    if let Some(data) = data {
        let token_data: TokenData = serde_json::from_str(&data)?;
        Ok(Some(token_data))
    } else {
        Ok(None)
    }
}

pub async fn start_tcp_server(tokens: SharedTokens, redis_client: RedisClient) {
    let listener = TcpListener::bind("0.0.0.0:8081").await.unwrap();
    println!("TCP server running on port 8081");

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        let tokens = tokens.clone();
        let redis_client = redis_client.clone();

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            loop {
                match timeout(Duration::from_secs(30), socket.read(&mut buf)).await {
                    Ok(Ok(n)) if n == 0 => return, // connection was closed
                    Ok(Ok(n)) => {
                        let token = String::from_utf8_lossy(&buf[..n]).to_string();
                        let token_data = get_token(&redis_client, &token).await.unwrap();
                        if let Some(token_data) = token_data {
                            if token_data.expiry_time > SystemTime::now() {
                                socket.write_all(b"Authenticated").await.unwrap();
                            } else if let Some(refresh_token) = &token_data.refresh_token {
                                // Refresh the token
                                // Here you would add the code to refresh the token
                                // For now, we will just print a message
                                println!("Token expired, refreshing...");
                                // Update the token data in the store after refreshing
                            } else {
                                socket.write_all(b"Not Authenticated").await.unwrap();
                            }
                        } else {
                            socket.write_all(b"Not Authenticated").await.unwrap();
                        }
                    }
                    Ok(Err(e)) => {
                        println!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                    Err(_) => {
                        println!("heartbeat timeout");
                        return;
                    }
                }
            }
        });
    }
}
