use axum::middleware::from_fn_with_state;
use axum::{Router, routing::get};

use super::revalidate::revalidate;
use crate::app_state::AppState;
use crate::handlers::search;

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(search::search_pka))
        .route_layer(from_fn_with_state(state.clone(), revalidate))
}
