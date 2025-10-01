use redis::{AsyncCommands, Client, RedisResult};
use tracing::{info, warn, error, debug, instrument};

use crate::database::AccountUpdate;

#[derive(Debug)]
pub struct RedisCache {
    client: Client,
}

impl RedisCache {
    #[instrument(skip(redis_url))]
    pub async fn new(redis_url: &str) -> RedisResult<Self> {
        debug!("Connecting to Redis");
        let client = Client::open(redis_url)?;

        // Test the connection
        let mut conn = client.get_async_connection().await?;
        let ping_response: String = redis::cmd("PING").query_async(&mut conn).await?;

        info!("Redis connection established, ping response: {}", ping_response);
        Ok(Self { client })
    }

    #[instrument(skip(self, account), fields(pubkey = %pubkey))]
    pub async fn set_account(&self, pubkey: &str, account: &AccountUpdate) -> RedisResult<()> {
        debug!(pubkey = %pubkey, "🔴 Setting account in Redis cache");

        let mut conn = self.client.get_async_connection().await?;
        let key = format!("account:{}", pubkey);
        let account_json = serde_json::to_string(account).map_err(|e| {
            error!(pubkey = %pubkey, error = %e, "❌ JSON serialization failed for Redis cache");
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "JSON serialization failed",
                e.to_string(),
            ))
        })?;

        // Explicit type annotation for Redis return value (TTL: 1 hour)
        let _: () = conn.set_ex(&key, account_json, 3600).await?;

        info!(
            pubkey = %pubkey,
            ttl_seconds = 3600,
            account_type = %account.account_type,
            "✅ Account cached in Redis successfully"
        );

        Ok(())
    }

    #[instrument(skip(self), fields(pubkey = %pubkey))]
    pub async fn get_account(&self, pubkey: &str) -> RedisResult<Option<AccountUpdate>> {
        debug!(pubkey = %pubkey, "🔍 Retrieving account from Redis cache");

        let mut conn = self.client.get_async_connection().await?;
        let key = format!("account:{}", pubkey);

        // Get returns Option<String> - None if key doesn't exist
        let account_json: Option<String> = conn.get(&key).await?;

        match account_json {
            Some(json_str) => {
                let account: AccountUpdate = serde_json::from_str(&json_str).map_err(|e| {
                    error!(pubkey = %pubkey, error = %e, "❌ JSON deserialization failed for Redis cache");
                    redis::RedisError::from((
                        redis::ErrorKind::TypeError,
                        "JSON deserialization failed",
                        e.to_string(),
                    ))
                })?;

                info!(
                    pubkey = %pubkey,
                    account_type = %account.account_type,
                    id = account.id,
                    "✅ Account retrieved from Redis cache successfully"
                );

                Ok(Some(account))
            }
            None => {
                debug!(pubkey = %pubkey, "🔍 Account not found in Redis cache");
                Ok(None)
            }
        }
    }

    pub async fn delete_account(&self, pubkey: &str) -> RedisResult<bool> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("account:{}", pubkey);
        let deleted: bool = conn.del(&key).await?;
        Ok(deleted)
    }

    pub async fn exists_account(&self, pubkey: &str) -> RedisResult<bool> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("account:{}", pubkey);
        let exists: bool = conn.exists(&key).await?;
        Ok(exists)
    }

    pub async fn get_account_ttl(&self, pubkey: &str) -> RedisResult<i64> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("account:{}", pubkey);
        let ttl: i64 = conn.ttl(&key).await?;
        Ok(ttl)
    }
}
