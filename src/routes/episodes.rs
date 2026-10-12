use axum::middleware::from_fn_with_state;
use axum::{Router, routing::get};

use super::revalidate::revalidate;
use crate::app_state::AppState;
use crate::handlers::episode;

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(episode::all_pka_episodes))
        .route("/{number}", get(episode::watch_pka_episode))
        .route(
            "/{number}/youtube-link",
            get(episode::find_pka_episode_youtube_link),
        )
        .route("/latest", get(episode::latest_pka_episode))
        // Only the routes above: a random pick must never be "not modified".
        .route_layer(from_fn_with_state(state.clone(), revalidate))
        .route("/random", get(episode::random_pka_episode))
}
