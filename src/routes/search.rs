use axum::{routing::post, Router};

use crate::app_state::AppState;
use crate::handlers::search;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/episodes", post(search::search_pka_episode))
        .route("/events", post(search::search_pka_event))
}
