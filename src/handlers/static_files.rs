use float_ord::FloatOrd;

use axum::extract::State;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;

use crate::app_state::AppState;
use crate::conduit::sqlite::pka_episode;
use crate::models::errors::ApiError;
use crate::models::sitemap_xml::{SiteMap, Url};

#[utoipa::path(
    get,
    path = "/robots.txt",
    responses(
        (status = 200, description = "Robots rules", content_type = "text/plain", body = String),
        (status = 500, description = "Internal server error", body = crate::models::errors::ErrorResponseBody)
    ),
    tag = "Static"
)]
pub async fn robots_txt() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        )],
        "User-agent: *\nDisallow: ".to_owned(),
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
        ),
        (status = 500, description = "Internal server error", body = crate::models::errors::ErrorResponseBody)
    ),
    tag = "Static"
)]
pub async fn sitemap_xml(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let mut res = pka_episode::all(state.db.as_ref())
        .await
        .map_err(ApiError::from)?;

    res.sort_by_key(|a| FloatOrd(a.number()));

    let mut urls = vec![
        Url::new(
            "https://www.pkaindex.com/",
            None,
            Some("weekly"),
            Some("1.0"),
        ),
        Url::new(
            "https://www.pkaindex.com/watch",
            None,
            Some("weekly"),
            Some("1.0"),
        ),
        Url::new(
            "https://www.pkaindex.com/watch/latest",
            None,
            Some("weekly"),
            Some("1.0"),
        ),
        Url::new(
            "https://www.pkaindex.com/episodes",
            None,
            Some("weekly"),
            Some("0.9"),
        ),
        Url::new(
            "https://www.pkaindex.com/events",
            None,
            Some("weekly"),
            Some("0.9"),
        ),
        Url::new(
            "https://www.pkaindex.com/watch/random",
            None,
            Some("weekly"),
            Some("0.8"),
        ),
    ];

    res.into_iter()
        .map(|p| {
            Url::new(
                format!("https://www.pkaindex.com/watch/{}", p.number()).as_str(),
                None,
                Some("weekly"),
                Some("0.7"),
            )
        })
        .for_each(|u| {
            urls.push(u);
        });

    let sitemap = SiteMap::from_urls(urls);
    let xml = sitemap.to_xml_string()?;

    Ok((
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/xml"),
        )],
        xml,
    ))
}
