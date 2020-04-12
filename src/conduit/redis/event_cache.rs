use redis::RedisError;

use crate::models::search::PkaEventSearchResult;
use crate::redis_db::RedisDb;

pub async fn set(
    redis: &RedisDb,
    redis_tag: &str,
    key: String,
    value: Vec<PkaEventSearchResult>,
) -> Result<(), RedisError> {
    redis.set(redis_tag.to_string(), key, value).await?;

    Ok(())
}

pub async fn get(
    redis: &RedisDb,
    redis_tag: &str,
    key: String,
) -> Result<Vec<PkaEventSearchResult>, RedisError> {
    let res: Vec<PkaEventSearchResult> = redis.get(redis_tag.to_string(), key).await?;

    Ok(res)
}
