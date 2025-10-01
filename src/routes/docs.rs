use std::sync::Arc;

use axum::routing::get;
use axum::{Json, Router};

use crate::app_state::AppState;
use crate::docs;

pub fn router() -> Router<AppState> {
    let openapi = Arc::new(docs::openapi());

    Router::new().route("/openapi.json", {
        get(move || async move { Json(openapi) })
    })
}
