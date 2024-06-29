use super::data_redis::{AuthRequest, SharedTokens, TokenData, TokenResponse};
use redis::AsyncCommands;
use std::time::SystemTime;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) async fn store_token(
    redis_client: &redis::Client,
    key: &str,
    token_data: &TokenData,
) -> redis::RedisResult<()> {
    let mut conn = redis_client.get_async_connection().await?;
    let _: () = conn.set(key, serde_json::to_string(token_data)?).await?;
    let _: () = conn
        .expire_at(
            key,
            token_data
                .expiry_time
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs() as usize,
        )
        .await?;
    Ok(())
}

async fn get_token(
    redis_client: &redis::Client,
    key: &str,
) -> redis::RedisResult<Option<TokenData>> {
    let mut conn = redis_client.get_async_connection().await?;
    let data: Option<String> = conn.get(key).await?;
    if let Some(data) = data {
        let token_data: TokenData = serde_json::from_str(&data)?;
        Ok(Some(token_data))
    } else {
        Ok(None)
    }
}