use chrono::{DateTime, Utc};
use compact_str::CompactString;
use serde::{de, Deserialize, Deserializer, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItemsResponse {
    pub kind: CompactString,
    pub etag: CompactString,
    pub next_page_token: CompactString,
    pub items: Vec<PlaylistItem>,
    pub page_info: PageInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItem {
    pub kind: CompactString,
    pub etag: CompactString,
    pub id: CompactString,
    pub snippet: Snippet,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    #[serde(deserialize_with = "published_at_timestamp")]
    pub published_at: i64,
    pub channel_id: CompactString,
    pub title: CompactString,
    pub description: CompactString,
    pub thumbnails: Thumbnails,
    pub channel_title: CompactString,
    pub playlist_id: CompactString,
    pub position: i64,
    pub resource_id: ResourceId,
    pub video_owner_channel_title: CompactString,
    pub video_owner_channel_id: CompactString,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnails {
    pub default: Default,
    pub medium: Medium,
    pub high: High,
    pub standard: Standard,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Default {
    pub url: CompactString,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Medium {
    pub url: CompactString,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct High {
    pub url: CompactString,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Standard {
    pub url: CompactString,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Maxres {
    pub url: CompactString,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceId {
    pub kind: CompactString,
    pub video_id: CompactString,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub total_results: i64,
    pub results_per_page: i64,
}

fn published_at_timestamp<'de, D>(deserializer: D) -> std::result::Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <CompactString>::deserialize(deserializer)?;
    let published = s
        .parse::<DateTime<Utc>>()
        .map_err(de::Error::custom)?
        .timestamp();

    Ok(published)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn published_at_timestamp() {
        let published = "2022-10-29T12:54:45Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(published.timestamp(), 1667048085);
    }
}
