use axum::http::StatusCode;
use axum::routing::{any, get, post};
use axum::Router;

use crate::app_state::AppState;
use crate::handlers::{episode, event, search, static_files};
use crate::models::errors::ApiError;

pub fn build_router() -> Router<AppState> {
    let episodes = Router::new()
        .route("/{number}", get(episode::watch_pka_episode))
        .route(
            "/{number}/youtube-link",
            get(episode::find_pka_episode_youtube_link),
        )
        .route("/latest", get(episode::latest_pka_episode))
        .route("/random", get(episode::random_pka_episode));

    let events = Router::new().route("/random", get(event::random_pka_event));

    let search_router = Router::new()
        .route("/episodes", post(search::search_pka_episode))
        .route("/events", post(search::search_pka_event));

    let api = Router::new()
        .nest("/episodes", episodes)
        .nest("/events", events)
        .nest("/search", search_router);

    Router::new()
        .nest("/api/v1", api)
        .route("/robots.txt", get(static_files::robots_txt))
        .route("/sitemap.xml", get(static_files::sitemap_xml))
        .fallback(any(not_found))
}

async fn not_found() -> ApiError {
    ApiError::new("Page not found.", StatusCode::NOT_FOUND)
}
