//! The sitemap: the home page and every episode's watch page, each dated by
//! the newest upload it shows.

use std::iter;

use chrono::DateTime;

use crate::catalog::CatalogEpisode;
use crate::models::sitemap_xml::{SiteMap, Url};

const SITE: &str = "https://www.pkaindex.com";

fn date(unix_seconds: i64) -> Option<String> {
    DateTime::from_timestamp(unix_seconds, 0).map(|time| time.format("%Y-%m-%d").to_string())
}

/// Episodes newest first, as the catalog holds them.
pub fn render(episodes: &[CatalogEpisode]) -> anyhow::Result<String> {
    let home = Url::new(
        format!("{SITE}/"),
        episodes
            .first()
            .and_then(|newest| date(newest.episode.upload_date)),
        None,
        None,
    );
    let watch_pages = episodes.iter().map(|episode| {
        Url::new(
            format!("{SITE}/watch/{}", episode.episode.number),
            date(episode.episode.upload_date),
            None,
            None,
        )
    });

    SiteMap::from_urls(iter::once(home).chain(watch_pages).collect()).to_xml_string()
}

#[cfg(test)]
mod tests {
    use crate::catalog::test_data::{catalog, episode};

    #[test]
    fn test_lists_home_and_watch_pages_with_upload_dates() {
        let catalog = catalog(
            vec![episode(524.1, "Part 1"), episode(825.0, "PKA 825")],
            Vec::new(),
        );
        let xml = String::from_utf8(catalog.sitemap_xml.to_vec()).unwrap();

        // 1_700_000_000 plus 825 days, and plus 524.1 days.
        assert!(xml.contains(
            "<url><loc>https://www.pkaindex.com/</loc><lastmod>2026-02-16</lastmod></url>"
        ));
        assert!(xml.contains(
            "<url><loc>https://www.pkaindex.com/watch/825</loc><lastmod>2026-02-16</lastmod></url>"
        ));
        assert!(xml.contains(
            "<loc>https://www.pkaindex.com/watch/524.1</loc><lastmod>2025-04-22</lastmod>"
        ));
        assert_eq!(xml.matches("<url>").count(), 3);
    }
}
