use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use compact_str::CompactString;

use crate::app_state::AppState;
use crate::extractors::AppPath;
use crate::handlers::NO_STORE;
use crate::models::errors::{ApiError, ErrorResponseBody};
use crate::models::pka_episode_with_all::PkaEpisodeWithAll;
use crate::models::search::EpisodeSummary;
use crate::models::success_response::SuccessResponse;

fn not_found() -> ApiError {
    ApiError::new("That episode isn't in the index.", StatusCode::NOT_FOUND)
}

#[utoipa::path(
    get,
    path = "/api/v1/episodes",
    responses(
        (status = 200, description = "Every episode, newest first", body = SuccessResponse<Vec<EpisodeSummary>>)
    ),
    tag = "Episodes"
)]
pub async fn all_pka_episodes(State(state): State<AppState>, request: HeaderMap) -> Response {
    // Precompressed once per snapshot, at a level per-request compression
    // couldn't afford; the compression layer passes an encoded body through.
    let (encoding, body) = state.catalog.get().episodes_json.negotiate(&request);
    let mut response = (
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            ),
            (header::VARY, HeaderValue::from_static("accept-encoding")),
        ],
        body,
    )
        .into_response();

    if let Some(encoding) = encoding {
        response
            .headers_mut()
            .insert(header::CONTENT_ENCODING, encoding);
    }

    response
}

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
        (status = 404, description = "Episode not found", body = ErrorResponseBody)
    ),
    tag = "Episodes"
)]
pub async fn watch_pka_episode(
    AppPath(number): AppPath<f32>,
    State(state): State<AppState>,
) -> Result<SuccessResponse<PkaEpisodeWithAll>, ApiError> {
    let catalog = state.catalog.get();
    let episode = catalog.episode(number).ok_or_else(not_found)?;

    Ok(SuccessResponse::new(catalog.with_all(episode)))
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
        (status = 404, description = "Episode not found", body = ErrorResponseBody)
    ),
    tag = "Episodes"
)]
pub async fn find_pka_episode_youtube_link(
    AppPath(number): AppPath<f32>,
    State(state): State<AppState>,
) -> Result<SuccessResponse<CompactString>, ApiError> {
    let catalog = state.catalog.get();
    let episode = catalog.episode(number).ok_or_else(not_found)?;

    Ok(SuccessResponse::new(episode.episode.youtube_link.clone()))
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
        (status = 404, description = "No episodes yet", body = ErrorResponseBody)
    ),
    tag = "Episodes"
)]
pub async fn latest_pka_episode(
    State(state): State<AppState>,
) -> Result<SuccessResponse<PkaEpisodeWithAll>, ApiError> {
    let catalog = state.catalog.get();
    let episode = catalog.latest().ok_or_else(not_found)?;

    Ok(SuccessResponse::new(catalog.with_all(episode)))
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
        (status = 404, description = "No episodes yet", body = ErrorResponseBody)
    ),
    tag = "Episodes"
)]
pub async fn random_pka_episode(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let catalog = state.catalog.get();
    let episode = catalog.random_episode().ok_or_else(not_found)?;

    Ok((NO_STORE, SuccessResponse::new(catalog.with_all(episode))))
}
