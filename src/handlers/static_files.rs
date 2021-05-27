use std::sync::Arc;

use float_ord::FloatOrd;
use warp::Rejection;

use crate::conduit::sqlite::pka_episode;
use crate::models::errors::ApiError;
use crate::models::sitemap_xml::{SiteMap, Url};
use crate::Repo;

pub async fn robots_txt() -> Result<impl warp::Reply, Rejection> {
    let robots = "User-agent: *\nDisallow: ".to_owned();

    Ok(robots)
}

pub async fn sitemap_xml(state: Arc<Repo>) -> Result<impl warp::Reply, Rejection> {
    let mut res = pka_episode::all(&state).await.map_err(ApiError::from)?;

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

    let response =
        warp::reply::with_header(sitemap.to_xml_string()?, "content-type", "application/xml");

    Ok(response)
}
