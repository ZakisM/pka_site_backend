use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "urlset")]
pub struct SiteMap {
    pub xmlns: String,
    #[serde(rename = "xmlns:image")]
    pub xmlns_image: String,
    #[serde(rename = "xmlns:video")]
    pub xmlns_video: String,
    #[serde(rename = "url")]
    pub urls: Vec<Url>,
}

impl SiteMap {
    pub fn from_urls(urls: Vec<Url>) -> Self {
        Self {
            xmlns: "http://www.sitemaps.org/schemas/sitemap/0.9".to_string(),
            xmlns_image: "http://www.google.com/schemas/sitemap-image/1.1".to_string(),
            xmlns_video: "http://www.google.com/schemas/sitemap-video/1.1".to_string(),
            urls,
        }
    }

    #[allow(unused)]
    pub fn urls(&self) -> &Vec<Url> {
        &self.urls
    }

    pub fn to_xml_string(&self) -> Result<String> {
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
        let xml_body = quick_xml::se::to_string(&self)?;

        xml.push_str(&xml_body);

        Ok(xml)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Url {
    pub loc: XmlVal,
    #[serde(rename = "lastmod")]
    pub last_mod: Option<XmlVal>,
    #[serde(rename = "changefreq")]
    pub change_freq: Option<XmlVal>,
    pub priority: Option<XmlVal>,
}

impl Url {
    pub fn new<T: AsRef<str>>(
        loc: T,
        last_mod: Option<T>,
        change_freq: Option<T>,
        priority: Option<T>,
    ) -> Self {
        Url {
            loc: XmlVal(loc.as_ref().to_owned()),
            last_mod: last_mod.map(|s| XmlVal(s.as_ref().to_owned())),
            change_freq: change_freq.map(|s| XmlVal(s.as_ref().to_owned())),
            priority: priority.map(|s| XmlVal(s.as_ref().to_owned())),
        }
    }

    #[allow(unused)]
    pub fn loc(&self) -> &str {
        &self.loc.0
    }

    #[allow(unused)]
    pub fn last_mod(&self) -> Option<&str> {
        self.last_mod.as_ref().map(|v| v.0.as_str())
    }

    #[allow(unused)]
    pub fn change_freq(&self) -> Option<&str> {
        self.change_freq.as_ref().map(|v| v.0.as_str())
    }

    #[allow(unused)]
    pub fn priority(&self) -> Option<&str> {
        self.priority.as_ref().map(|v| v.0.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct XmlVal(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    const SITEMAP_RAW: &str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>
<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\" xmlns:image=\"http://www.google.com/schemas/sitemap-image/1.1\" xmlns:video=\"http://www.google.com/schemas/sitemap-video/1.1\"><url><loc>https://example.com/</loc><lastmod>2005-01-01</lastmod><changefreq>monthly</changefreq><priority>0.8</priority></url><url><loc>https://example2.com/</loc><lastmod>2025-01-01</lastmod><changefreq>weekly</changefreq><priority>0.2</priority></url></urlset>";

    #[test]
    fn test_write_sitemap() {
        let sitemap = SiteMap::from_urls(vec![
            Url::new(
                "https://example.com/",
                Some("2005-01-01"),
                Some("monthly"),
                Some("0.8"),
            ),
            Url::new(
                "https://example2.com/",
                Some("2025-01-01"),
                Some("weekly"),
                Some("0.2"),
            ),
        ]);

        let sitemap_r = sitemap.to_xml_string().expect("Failed to write sitemap");

        assert_eq!(&sitemap_r, SITEMAP_RAW);
    }

    #[test]
    fn test_read_sitemap() {
        let sitemap: SiteMap =
            quick_xml::de::from_str(&SITEMAP_RAW).expect("Failed to read sitemap");

        let first_url = sitemap.urls.first().unwrap();

        assert_eq!(&first_url.loc.0, "https://example.com/");
        assert_eq!(first_url.last_mod(), Some("2005-01-01"));
        assert_eq!(first_url.change_freq(), Some("monthly"));
        assert_eq!(first_url.priority(), Some("0.8"));

        let second_url = sitemap.urls.get(1).unwrap();

        assert_eq!(&second_url.loc.0, "https://example2.com/");
        assert_eq!(second_url.last_mod(), Some("2025-01-01"));
        assert_eq!(second_url.change_freq(), Some("weekly"));
        assert_eq!(second_url.priority(), Some("0.2"));
    }
}
