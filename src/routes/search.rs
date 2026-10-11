use axum::{routing::get, Router};

use crate::app_state::AppState;
use crate::handlers::search;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(search::search_pka))
}
