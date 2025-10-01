mod episodes;
mod events;
mod search;
mod static_assets;

use axum::http::StatusCode;
use axum::routing::any;
use axum::Router;

use crate::app_state::AppState;
use crate::models::errors::ApiError;

pub fn build_router() -> Router<AppState> {
    let api = Router::new()
        .nest("/episodes", episodes::router())
        .nest("/events", events::router())
        .nest("/search", search::router());

    Router::new()
        .nest("/api/v1", api)
        .merge(static_assets::router())
        .fallback(any(not_found))
}

async fn not_found() -> ApiError {
    ApiError::new("Page not found.", StatusCode::NOT_FOUND)
}
