use axum::extract::State;
use axum::http::StatusCode;

use compact_str::CompactString;

use crate::app_state::AppState;
use crate::conduit::sqlite::pka_episode;
use crate::conduit::sqlite::pka_episode::find_youtube_link;
use crate::extractors::AppPath;
use crate::models::errors::ApiError;
use crate::models::pka_episode_with_all::PkaEpisodeWithAll;
use crate::models::success_response::SuccessResponse;

pub async fn watch_pka_episode(
    AppPath(number): AppPath<f32>,
    State(state): State<AppState>,
) -> Result<SuccessResponse<PkaEpisodeWithAll>, ApiError> {
    let res = pka_episode::find_with_all(state.db.as_ref(), number)
        .await
        .map_err(ApiError::from)?;

    Ok(SuccessResponse::new(res))
}

pub async fn find_pka_episode_youtube_link(
    AppPath(number): AppPath<f32>,
    State(state): State<AppState>,
) -> Result<SuccessResponse<CompactString>, ApiError> {
    let res = find_youtube_link(state.db.as_ref(), number)
        .await
        .map_err(|_| ApiError::new("Couldn't find episode number", StatusCode::NOT_FOUND))?;

    Ok(SuccessResponse::new(res))
}

pub async fn latest_pka_episode(
    State(state): State<AppState>,
) -> Result<SuccessResponse<PkaEpisodeWithAll>, ApiError> {
    let latest_episode_number = pka_episode::latest(state.db.as_ref())
        .await
        .map_err(|_| ApiError::new_internal_error("Couldn't get latest episode number."))?;

    let res = pka_episode::find_with_all(state.db.as_ref(), latest_episode_number)
        .await
        .map_err(ApiError::from)?;

    Ok(SuccessResponse::new(res))
}

pub async fn random_pka_episode(
    State(state): State<AppState>,
) -> Result<SuccessResponse<PkaEpisodeWithAll>, ApiError> {
    let random_episode_number = pka_episode::random(state.db.as_ref())
        .await
        .map_err(|_| ApiError::new_internal_error("Couldn't get random episode number."))?;

    let res = pka_episode::find_with_all(state.db.as_ref(), random_episode_number)
        .await
        .map_err(ApiError::from)?;

    Ok(SuccessResponse::new(res))
}
