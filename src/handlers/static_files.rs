use axum::extract::State;
use axum::http::{HeaderName, HeaderValue, header};
use axum::response::IntoResponse;

use crate::app_state::AppState;

const STATIC_CACHE: (HeaderName, HeaderValue) = (
    header::CACHE_CONTROL,
    HeaderValue::from_static("public, max-age=3600"),
);

#[utoipa::path(
    get,
    path = "/robots.txt",
    responses(
        (status = 200, description = "Robots rules", content_type = "text/plain", body = String)
    ),
    tag = "Static"
)]
pub async fn robots_txt() -> impl IntoResponse {
    (
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            ),
            STATIC_CACHE,
        ],
        "User-agent: *\nDisallow:\nSitemap: https://www.pkaindex.com/sitemap.xml\n",
    )
}

#[utoipa::path(
    get,
    path = "/sitemap.xml",
    responses(
        (
            status = 200,
            description = "Sitemap XML",
            content_type = "application/xml",
            body = String
        )
    ),
    tag = "Static"
)]
pub async fn sitemap_xml(State(state): State<AppState>) -> impl IntoResponse {
    (
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/xml"),
            ),
            STATIC_CACHE,
        ],
        state.catalog.get().sitemap_xml.clone(),
    )
}
