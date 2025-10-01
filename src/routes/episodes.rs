use axum::{routing::get, Router};

use crate::app_state::AppState;
use crate::handlers::episode;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{number}", get(episode::watch_pka_episode))
        .route(
            "/{number}/youtube-link",
            get(episode::find_pka_episode_youtube_link),
        )
        .route("/latest", get(episode::latest_pka_episode))
        .route("/random", get(episode::random_pka_episode))
}
