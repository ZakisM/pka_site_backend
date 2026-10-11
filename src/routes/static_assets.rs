use axum::{Router, routing::get};

use crate::app_state::AppState;
use crate::handlers::static_files;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/robots.txt", get(static_files::robots_txt))
        .route("/sitemap.xml", get(static_files::sitemap_xml))
}
