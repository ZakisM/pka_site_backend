use float_ord::FloatOrd;

use anyhow::Context;
use axum::extract::State;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;

use crate::app_state::AppState;
use crate::conduit::sqlite::pka_episode;
use crate::models::errors::{ApiError, ErrorResponseBody};
use crate::models::sitemap_xml::{SiteMap, Url};

#[utoipa::path(
    get,
    path = "/robots.txt",
    responses(
        (status = 200, description = "Robots rules", content_type = "text/plain", body = String),
        (status = 500, description = "Internal server error", body = ErrorResponseBody)
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
        (status = 500, description = "Internal server error", body = ErrorResponseBody)
    ),
    tag = "Static"
)]
pub async fn sitemap_xml(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let mut res = pka_episode::all(state.db.as_ref())
        .await
        .context("Failed to load episodes for sitemap")?;

    res.sort_by_key(|a| FloatOrd(a.number()));

    type Type<'a> = &'a [(&'a str, Option<&'a str>, Option<&'a str>, Option<&'a str>)];

    const STATIC_URLS: Type = &[
        (
            "https://www.pkaindex.com/",
            None,
            Some("weekly"),
            Some("1.0"),
        ),
        (
            "https://www.pkaindex.com/watch",
            None,
            Some("weekly"),
            Some("1.0"),
        ),
        (
            "https://www.pkaindex.com/watch/latest",
            None,
            Some("weekly"),
            Some("1.0"),
        ),
        (
            "https://www.pkaindex.com/episodes",
            None,
            Some("weekly"),
            Some("0.9"),
        ),
        (
            "https://www.pkaindex.com/events",
            None,
            Some("weekly"),
            Some("0.9"),
        ),
        (
            "https://www.pkaindex.com/watch/random",
            None,
            Some("weekly"),
            Some("0.8"),
        ),
    ];

    let mut urls = STATIC_URLS
        .iter()
        .map(|(loc, last_mod, change_freq, priority)| {
            Url::new(
                *loc,
                last_mod.map(str::to_owned),
                change_freq.map(str::to_owned),
                priority.map(str::to_owned),
            )
        })
        .collect::<Vec<_>>();

    urls.extend(res.into_iter().map(|p| {
        Url::new(
            format!("https://www.pkaindex.com/watch/{}", p.number()),
            None,
            Some("weekly".to_owned()),
            Some("0.7".to_owned()),
        )
    }));

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
