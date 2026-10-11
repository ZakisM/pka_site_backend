use axum::http::{HeaderName, HeaderValue, header};

pub mod episode;
pub mod event;
pub mod related;
pub mod search;
pub mod static_files;

/// For answers that should differ on every request: the random picks.
pub const NO_STORE: [(HeaderName, HeaderValue); 1] =
    [(header::CACHE_CONTROL, HeaderValue::from_static("no-store"))];
