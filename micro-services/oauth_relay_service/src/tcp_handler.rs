use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio::time::{timeout, Duration};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use rusqlite::Connection;

type SharedTokens = Arc<Mutex<HashMap<String, TokenData>>>;

#[derive(Clone, Debug)]
pub struct TokenData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub expiry_time: SystemTime,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn get_token_redis(redis_client: &RedisClient, key: &str) -> redis::RedisResult<Option<TokenData>> {
    let mut conn = redis_client.get_async_connection().await?;
    let data: Option<String> = conn.get(key).await?;
    if let Some(data) = data {
        let token_data: TokenData = serde_json::from_str(&data)?;
        Ok(Some(token_data))
    } else {
        Ok(None)
    }
}


pub async fn start_tcp_server_redis(tokens: SharedTokens, redis_client: RedisClient) {
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
                        let token_data = get_token_redis(&redis_client, &token).await.unwrap();
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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn get_token_postgres(client: &PgClient, client_id: &str) -> Result<Option<TokenData>, Error> {
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

pub async fn start_tcp_server_postgres(tokens: SharedTokens, pg_client: PgClient) {
    let listener = TcpListener::bind("0.0.0.0:8081").await.unwrap();
    println!("TCP server running on port 8081");

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        let tokens = tokens.clone();
        let pg_client = pg_client.clone();

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            loop {
                match timeout(Duration::from_secs(30), socket.read(&mut buf)).await {
                    Ok(Ok(n)) if n == 0 => return, // connection was closed
                    Ok(Ok(n)) => {
                        let token = String::from_utf8_lossy(&buf[..n]).to_string();
                        let token_data = get_token_postgres(&pg_client, &token).await.unwrap();
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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn get_token_sqlite(conn: &Connection, client_id: &str) -> Result<Option<TokenData>, Error> {
    let mut stmt = conn.prepare("SELECT access_token, refresh_token, expires_in, expiry_time FROM tokens WHERE client_id = ?1")?;
    let token_iter = stmt.query_map([client_id], |row| {
        Ok(TokenData {
            access_token: row.get(0)?,
            refresh_token: row.get(1)?,
            expires_in: row.get(2)?,
            expiry_time: UNIX_EPOCH + std::time::Duration::from_secs(row.get::<_, i64>(3)? as u64),
        })
    })?;

    for token in token_iter {
        return Ok(Some(token?));
    }
    Ok(None)
}

pub async fn start_tcp_server_sqlite(tokens: SharedTokens, conn: Connection) {
    let listener = TcpListener::bind("0.0.0.0:8081").await.unwrap();
    println!("TCP server running on port 8081");

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        let tokens = tokens.clone();
        let conn = conn.clone();

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            loop {
                match timeout(Duration::from_secs(30), socket.read(&mut buf)).await {
                    Ok(Ok(n)) if n == 0 => return, // connection was closed
                    Ok(Ok(n)) => {
                        let token = String::from_utf8_lossy(&buf[..n]).to_string();
                        let token_data = get_token_sqlite(&conn, &token).unwrap();
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