use std::convert::Infallible;
use std::sync::Arc;

use reqwest::StatusCode;
use warp::Reply;

use crate::conduit::sqlite::pka_episode;
use crate::conduit::sqlite::pka_episode::find_youtube_link;
use crate::models::errors::ApiError;
use crate::models::success_response::SuccessResponse;
use crate::Repo;

pub async fn watch_pka_episode(
    number: f32,
    state: Arc<Repo>,
) -> Result<impl warp::Reply, Infallible> {
    Ok(find_with_all(&state, number).await)
}

pub async fn find_pka_episode_youtube_link(
    number: f32,
    state: Arc<Repo>,
) -> Result<impl warp::Reply, Infallible> {
    match find_youtube_link(&state, number).await {
        Ok(res) => Ok(SuccessResponse::new(res).into_response()),
        Err(e) => {
            error!("{}", e);
            // Should return Err once improves in Warp;
            Ok(
                ApiError::new("Couldn't find episode number.", StatusCode::NOT_FOUND)
                    .into_response(),
            )
        }
    }
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
