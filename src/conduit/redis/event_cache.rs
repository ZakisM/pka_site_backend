use redis::RedisError;

use crate::redis_db::RedisDb;

pub async fn set(
    redis: &RedisDb,
    redis_tag: &str,
    key: String,
    value: Vec<u8>,
) -> Result<(), RedisError> {
    redis.set(redis_tag.to_string(), key, value).await?;

    Ok(())
}

pub async fn get(redis: &RedisDb, redis_tag: &str, key: String) -> Result<Vec<u8>, RedisError> {
    let res: Vec<u8> = redis.get(redis_tag.to_string(), key).await?;

    Ok(res)
}
