use axum::body::Body;
use axum::extract::{rejection::JsonRejection, Json, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::app_state::AppState;
use crate::models::errors::ApiError;
use crate::models::search::SearchQuery;
use crate::search::pka_search::{search_episode, search_events};

pub async fn search_pka_episode(
    State(state): State<AppState>,
    payload: Result<Json<SearchQuery>, JsonRejection>,
) -> Result<Response, ApiError> {
    let Json(payload) = payload
        .map_err(|rejection| ApiError::new(rejection.body_text(), StatusCode::BAD_REQUEST))?;

    let res = search_episode(state.db.as_ref(), &payload.query).await?;

    Ok(binary_response(res))
}

pub async fn search_pka_event(
    State(state): State<AppState>,
    payload: Result<Json<SearchQuery>, JsonRejection>,
) -> Result<Response, ApiError> {
    let Json(payload) = payload
        .map_err(|rejection| ApiError::new(rejection.body_text(), StatusCode::BAD_REQUEST))?;

    let res = search_events(state.redis.as_ref(), &payload.query).await?;

    Ok(binary_response(res))
}

fn binary_response(body: Vec<u8>) -> Response {
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        )],
        Body::from(body),
    )
        .into_response()
}
