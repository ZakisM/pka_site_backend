use anyhow::{ensure, Context};
use bb8_redis::bb8::{self, Pool, PooledConnection};
use bb8_redis::redis::AsyncCommands;
use bb8_redis::RedisConnectionManager;
use tokio::time::{self, Duration};

pub struct RedisDb {
    connection_pool: Pool<RedisConnectionManager>,
}

impl RedisDb {
    const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);

    pub async fn new(redis_url: &str) -> crate::Result<Self> {
        let manager = RedisConnectionManager::new(redis_url)
            .context("Failed to create redis connection manager")?;

        let pool = bb8::Pool::builder()
            .build(manager)
            .await
            .context("Failed to build redis pool")?;

        // Verify we can obtain a connection up-front to fail fast on misconfiguration.
        Self::acquire_connection(&pool)
            .await
            .context("Failed to acquire initial redis connection")?;

        Ok(RedisDb {
            connection_pool: pool,
        })
    }

    pub async fn get(&self, redis_tag: String, key: String) -> anyhow::Result<Vec<u8>> {
        let mut conn = Self::acquire_connection(&self.connection_pool).await?;

        let key = format!("{}-{}", redis_tag, key);

        let value: Vec<u8> = conn
            .get(&key)
            .await
            .context("Failed to fetch redis value")?;

        ensure!(!value.is_empty(), "Redis returned empty vector");

        Ok(value)
    }

    pub async fn set(&self, redis_tag: String, key: String, value: &[u8]) -> anyhow::Result<()> {
        ensure!(!value.is_empty(), "Redis will not cache empty vector");

        let value = value.to_vec();

        let mut conn = Self::acquire_connection(&self.connection_pool).await?;

        let key = format!("{}-{}", redis_tag, key);

        conn.set_ex::<_, _, ()>(key, value, 30)
            .await
            .context("Failed to cache redis value")?;

        Ok(())
    }

    async fn acquire_connection(
        pool: &Pool<RedisConnectionManager>,
    ) -> anyhow::Result<PooledConnection<'_, RedisConnectionManager>> {
        let get_connection = pool.get();

        let connection = time::timeout(Self::CONNECTION_TIMEOUT, get_connection)
            .await
            .context("Failed to get redis connection")??;

        Ok(connection)
    }
}
