use axum::middleware::from_fn_with_state;
use axum::{Router, routing::get};

use super::revalidate::revalidate;
use crate::app_state::AppState;
use crate::handlers::{event, related};

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/related", get(related::related_pka_events))
        // Only the routes above: a random pick must never be "not modified".
        .route_layer(from_fn_with_state(state.clone(), revalidate))
        .route("/random", get(event::random_pka_event))
}
