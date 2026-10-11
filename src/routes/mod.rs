mod docs;
mod episodes;
mod events;
mod search;
mod static_assets;

use axum::Router;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware::map_response;
use axum::response::Response;
use axum::routing::any;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};

use crate::app_state::AppState;
use crate::models::errors::ApiError;

// The catalog changes when an episode is added, a few times a week, so a
// Browser (or a proxy in front) may reuse an answer for a few minutes.
const API_CACHE: HeaderValue = HeaderValue::from_static("public, max-age=300");

/// The whole HTTP application: every route, with CORS, compression and
/// Caching headers.
pub fn app(state: AppState, expose_openapi: bool) -> Router {
    let api = Router::new()
        .nest("/episodes", episodes::router())
        .nest("/events", events::router())
        .nest("/search", search::router())
        .layer(map_response(default_cache_control));

    let mut router = Router::new()
        .nest("/api/v1", api)
        .merge(static_assets::router())
        .fallback(any(not_found));

    if expose_openapi {
        router = router.merge(docs::router());
    }

    // A public, read-only API: any site may read it, without credentials.
    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_origin(Any);

    router
        .with_state(state)
        .layer(cors)
        // The episode list and broad searches are tens of kilobytes of JSON.
        .layer(CompressionLayer::new())
}

/// Successful API answers are cacheable unless the handler says otherwise
/// (the random picks do); errors never are.
async fn default_cache_control(mut response: Response) -> Response {
    if response.status().is_success() {
        response
            .headers_mut()
            .entry(header::CACHE_CONTROL)
            .or_insert(API_CACHE);
    }

    response
}

async fn not_found() -> ApiError {
    ApiError::new("Page not found.", StatusCode::NOT_FOUND)
}
