use std::convert::Infallible;
use std::sync::Arc;

use warp::Reply;

use crate::conduit::pka_episode;
use crate::models::errors::ApiError;
use crate::models::success_response::SuccessResponse;
use crate::Repo;

pub async fn all_pka_episodes(state: Arc<Repo>) -> Result<impl warp::Reply, Infallible> {
    match pka_episode::all(&state).await {
        Ok(res) => Ok(SuccessResponse::new(res).into_response()),
        Err(e) => Ok(ApiError::from(e).into_response()),
    }
}

pub async fn watch_pka_episode(
    number: f32,
    state: Arc<Repo>,
) -> Result<impl warp::Reply, Infallible> {
    Ok(find_with_all(&state, number).await)
}

pub async fn latest_pka_episode(state: Arc<Repo>) -> Result<impl warp::Reply, Infallible> {
    let latest_episode_number = match pka_episode::latest(&state).await {
        Ok(n) => n,
        Err(_) => {
            return Ok(
                ApiError::new_internal_error("Couldn't get latest episode number.").into_response(),
            );
        }
    };

    Ok(find_with_all(&state, latest_episode_number).await)
}

async fn find_with_all(state: &Repo, number: f32) -> warp::reply::Response {
    match pka_episode::find_with_all(&state, number).await {
        Ok(res) => SuccessResponse::new(res).into_response(),
        Err(e) => ApiError::from(e).into_response(),
    }
}
