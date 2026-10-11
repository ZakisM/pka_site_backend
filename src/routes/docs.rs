use axum::routing::get;
use axum::{Json, Router};

use crate::app_state::AppState;
use crate::docs;

/// The OpenAPI document, for generating clients or browsing in any OpenAPI
/// Viewer. Only served when `EXPOSE_OPENAPI` is set.
pub fn router() -> Router<AppState> {
    let openapi = Json(docs::openapi());

    Router::new().route("/openapi.json", get(move || async move { openapi }))
}
