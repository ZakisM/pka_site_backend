use axum::extract::State;
use axum::http::StatusCode;

use anyhow::Context;
use compact_str::CompactString;

use crate::app_state::AppState;
use crate::conduit::sqlite::pka_episode;
use crate::conduit::sqlite::pka_episode::find_youtube_link;
use crate::extractors::AppPath;
use crate::models::errors::{ApiError, ErrorResponseBody};
use crate::models::pka_episode_with_all::PkaEpisodeWithAll;
use crate::models::success_response::SuccessResponse;

#[utoipa::path(
    get,
    path = "/api/v1/episodes/{number}",
    params(("number" = f32, Path, description = "Episode number")),
    responses(
        (
            status = 200,
            description = "Episode details",
            body = SuccessResponse<PkaEpisodeWithAll>
        ),
        (status = 400, description = "Invalid episode number", body = ErrorResponseBody),
        (status = 404, description = "Episode not found", body = ErrorResponseBody),
        (status = 500, description = "Internal server error", body = ErrorResponseBody)
    ),
    tag = "Episodes"
)]
pub async fn watch_pka_episode(
    AppPath(number): AppPath<f32>,
    State(state): State<AppState>,
) -> Result<SuccessResponse<PkaEpisodeWithAll>, ApiError> {
    let res = pka_episode::find_with_all(state.db.as_ref(), number).await?;

    Ok(SuccessResponse::new(res))
}

#[utoipa::path(
    get,
    path = "/api/v1/episodes/{number}/youtube-link",
    params(("number" = f32, Path, description = "Episode number")),
    responses(
        (
            status = 200,
            description = "Episode youtube link",
            body = SuccessResponse<String>
        ),
        (status = 400, description = "Invalid episode number", body = ErrorResponseBody),
        (status = 404, description = "Episode not found", body = ErrorResponseBody),
        (status = 500, description = "Internal server error", body = ErrorResponseBody)
    ),
    tag = "Episodes"
)]
pub async fn find_pka_episode_youtube_link(
    AppPath(number): AppPath<f32>,
    State(state): State<AppState>,
) -> Result<SuccessResponse<CompactString>, ApiError> {
    let res = find_youtube_link(state.db.as_ref(), number)
        .await
        .map_err(|_| ApiError::new("Couldn't find episode number", StatusCode::NOT_FOUND))?;

    Ok(SuccessResponse::new(res))
}

#[utoipa::path(
    get,
    path = "/api/v1/episodes/latest",
    responses(
        (
            status = 200,
            description = "Latest episode",
            body = SuccessResponse<PkaEpisodeWithAll>
        ),
        (status = 500, description = "Internal server error", body = ErrorResponseBody)
    ),
    tag = "Episodes"
)]
pub async fn latest_pka_episode(
    State(state): State<AppState>,
) -> Result<SuccessResponse<PkaEpisodeWithAll>, ApiError> {
    let latest_episode_number = pka_episode::latest(state.db.as_ref())
        .await
        .context("Couldn't get latest episode number.")?;

    let res = pka_episode::find_with_all(state.db.as_ref(), latest_episode_number).await?;

    Ok(SuccessResponse::new(res))
}

#[utoipa::path(
    get,
    path = "/api/v1/episodes/random",
    responses(
        (
            status = 200,
            description = "Random episode",
            body = SuccessResponse<PkaEpisodeWithAll>
        ),
        (status = 500, description = "Internal server error", body = ErrorResponseBody)
    ),
    tag = "Episodes"
)]
pub async fn random_pka_episode(
    State(state): State<AppState>,
) -> Result<SuccessResponse<PkaEpisodeWithAll>, ApiError> {
    let random_episode_number = pka_episode::random(state.db.as_ref())
        .await
        .context("Couldn't get random episode number.")?;

    let res = pka_episode::find_with_all(state.db.as_ref(), random_episode_number).await?;

    Ok(SuccessResponse::new(res))
}
