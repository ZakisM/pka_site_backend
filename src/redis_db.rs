use anyhow::Context;
use bb8_redis::bb8::{self, Pool};
use bb8_redis::redis::AsyncCommands;
use bb8_redis::RedisConnectionManager;

use crate::models::errors::ApiError;

pub struct RedisDb {
    connection_pool: Pool<RedisConnectionManager>,
}

impl RedisDb {
    pub async fn new(redis_url: &str) -> crate::Result<Self> {
        let manager = RedisConnectionManager::new(redis_url)
            .context("Failed to create redis connection manager")?;

        let pool = bb8::Pool::builder()
            .build(manager)
            .await
            .context("Failed to build redis pool")?;

        // Verify we can obtain a connection up-front to fail fast on misconfiguration.
        pool.get()
            .await
            .context("Failed to acquire initial redis connection")?;

        Ok(RedisDb {
            connection_pool: pool,
        })
    }

    pub async fn get(&self, redis_tag: String, key: String) -> Result<Vec<u8>, ApiError> {
        let mut conn = self
            .connection_pool
            .get()
            .await
            .context("Failed to get redis connection")?;

        let key = format!("{}-{}", redis_tag, key);

        let value: Vec<u8> = conn.get(&key).await?;

        if value.is_empty() {
            Err(ApiError::new_internal_error(
                "Redis will not return empty vector.",
            ))
        } else {
            Ok(value)
        }
    }

    pub async fn set(&self, redis_tag: String, key: String, value: &[u8]) -> Result<(), ApiError> {
        if value.is_empty() {
            return Err(ApiError::new_internal_error(
                "Redis will not cache empty vector.",
            ));
        }

        let value = value.to_vec();

        let mut conn = self
            .connection_pool
            .get()
            .await
            .context("Failed to get redis connection")?;

        let key = format!("{}-{}", redis_tag, key);

        conn.set_ex::<_, _, ()>(key, value, 30).await?;

        Ok(())
    }
}
