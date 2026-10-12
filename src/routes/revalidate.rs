//! Conditional requests for answers drawn from the catalog. Every one of them
//! carries the catalog's version as its ETag, so once a browser's copy goes
//! stale it asks with `If-None-Match`, and while nothing has changed the
//! answer is an empty 304 rather than the whole body again. The random picks
//! are left out: they differ on every request by design.

use axum::extract::{Request, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use super::API_CACHE;
use crate::app_state::AppState;

// Weak: the same answer goes out in several compressions, which a strong tag
// would have to tell apart.
fn etag(version: u64) -> HeaderValue {
    HeaderValue::from_str(&format!("W/\"{version:016x}\"")).expect("a hex tag is a valid header")
}

// If-None-Match compares weakly, and may list several tags.
fn matches(if_none_match: &HeaderValue, etag: &HeaderValue) -> bool {
    let ours = etag.to_str().unwrap_or_default().trim_start_matches("W/");

    if_none_match.to_str().is_ok_and(|tags| {
        tags.split(',')
            .map(|tag| tag.trim().trim_start_matches("W/"))
            .any(|tag| tag == "*" || tag == ours)
    })
}

pub async fn revalidate(State(state): State<AppState>, request: Request, next: Next) -> Response {
    // Read before the handler: if the catalog is replaced meanwhile, the tag
    // is older than the answer, so the next request fetches again rather than
    // being told a stale copy is current.
    let etag = etag(state.catalog.get().version);
    let unchanged = request
        .headers()
        .get_all(header::IF_NONE_MATCH)
        .iter()
        .any(|value| matches(value, &etag));

    if unchanged {
        return (
            StatusCode::NOT_MODIFIED,
            [(header::ETAG, etag), (header::CACHE_CONTROL, API_CACHE)],
        )
            .into_response();
    }

    let mut response = next.run(request).await;

    if response.status().is_success() {
        response.headers_mut().insert(header::ETAG, etag);
    }

    response
}
