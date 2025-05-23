use std::ops::DerefMut;

use bb8_redis::bb8::Pool;
use bb8_redis::{bb8, redis::AsyncCommands, RedisConnectionManager};
use redis::RedisError;

use crate::models::errors::ApiError;

pub struct RedisDb {
    connection_pool: Pool<RedisConnectionManager>,
}

impl RedisDb {
    pub async fn new(redis_url: &str) -> crate::Result<Self> {
        Self::test_connection(redis_url)?;

        let manager = RedisConnectionManager::new(redis_url)
            .expect("Failed to create redis connection manager.");

        let pool = bb8::Pool::builder()
            .build(manager)
            .await
            .expect("Failed to build redis pool.");

        Ok(RedisDb {
            connection_pool: pool,
        })
    }

    fn test_connection(redis_url: &str) -> Result<(), RedisError> {
        let _ = redis::Client::open(redis_url)?.get_connection()?;
        Ok(())
    }

    pub async fn get(&self, redis_tag: String, key: String) -> Result<Vec<u8>, ApiError> {
        let pool = self.connection_pool.clone();

        tokio::spawn(async move {
            let mut conn = pool
                .get()
                .await
                .expect("Failed to get connection from pool.");

            let conn = conn.deref_mut();

            let key = format!("{}-{}", redis_tag, key);

            let value: Vec<u8> = conn.get(&key).await?;

            if value.is_empty() {
                Err(ApiError::new_internal_error(
                    "Redis will not return empty vector.",
                ))
            } else {
                Ok(value)
            }
        })
        .await
        .expect("Failed to run redis get task.")
    }

    pub async fn set(&self, redis_tag: String, key: String, value: &[u8]) -> Result<(), ApiError> {
        if value.is_empty() {
            return Err(ApiError::new_internal_error(
                "Redis will not cache empty vector.",
            ));
        }

        let value = value.to_vec();

        let pool = self.connection_pool.clone();

        tokio::spawn(async move {
            let mut conn = pool
                .get()
                .await
                .expect("Failed to get connection from pool.");

            let conn = conn.deref_mut();

            let key = format!("{}-{}", redis_tag, key);

            conn.set_ex::<_, _, ()>(key, value, 30).await?;

            Ok(())
        })
        .await
        .expect("Failed to run redis set task.")
    }
}
