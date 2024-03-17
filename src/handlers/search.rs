use std::sync::Arc;

use warp::Rejection;

use crate::models::errors::ApiError;
use crate::models::search::SearchQuery;
use crate::redis_db::RedisDb;
use crate::search::pka_search::{search_episode, search_events};
use crate::Repo;

pub async fn search_pka_episode(
    sq: SearchQuery,
    state: Arc<Repo>,
) -> Result<impl warp::Reply, Rejection> {
    let res = search_episode(&state, &sq.query)
        .await
        .map_err(ApiError::from)?;

    Ok(res)
}

pub async fn search_pka_event(
    sq: SearchQuery,
    redis: Arc<RedisDb>,
) -> Result<impl warp::Reply, Rejection> {
    let res = search_events(&redis, &sq.query)
        .await
        .map_err(ApiError::from)?;

    Ok(res)
}
