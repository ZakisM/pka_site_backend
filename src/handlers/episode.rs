use std::sync::Arc;

use warp::http::StatusCode;
use warp::Rejection;

use crate::conduit::sqlite::pka_episode;
use crate::conduit::sqlite::pka_episode::find_youtube_link;
use crate::models::errors::ApiError;
use crate::models::success_response::SuccessResponse;
use crate::Repo;

pub async fn watch_pka_episode(
    number: f32,
    state: Arc<Repo>,
) -> Result<impl warp::Reply, Rejection> {
    let res = pka_episode::find_with_all(&state, number)
        .await
        .map_err(ApiError::from)?;

    Ok(SuccessResponse::new(res))
}

pub async fn find_pka_episode_youtube_link(
    number: f32,
    state: Arc<Repo>,
) -> Result<impl warp::Reply, Rejection> {
    let res = find_youtube_link(&state, number)
        .await
        .map_err(|_| ApiError::new("Couldn't find episode number", StatusCode::NOT_FOUND))?;

    Ok(SuccessResponse::new(res))
}

pub async fn latest_pka_episode(state: Arc<Repo>) -> Result<impl warp::Reply, Rejection> {
    let latest_episode_number = pka_episode::latest(&state)
        .await
        .map_err(|_| ApiError::new_internal_error("Couldn't get latest episode number."))?;

    let res = pka_episode::find_with_all(&state, latest_episode_number)
        .await
        .map_err(ApiError::from)?;

    Ok(SuccessResponse::new(res))
}

pub async fn random_pka_episode(state: Arc<Repo>) -> Result<impl warp::Reply, Rejection> {
    let random_episode_number = pka_episode::random(&state)
        .await
        .map_err(|_| ApiError::new_internal_error("Couldn't get random episode number."))?;

    let res = pka_episode::find_with_all(&state, random_episode_number)
        .await
        .map_err(ApiError::from)?;

    Ok(SuccessResponse::new(res))
}
