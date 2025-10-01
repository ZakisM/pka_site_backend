use axum::{routing::get, Router};

use crate::app_state::AppState;
use crate::handlers::event;

pub fn router() -> Router<AppState> {
    Router::new().route("/random", get(event::random_pka_event))
}
