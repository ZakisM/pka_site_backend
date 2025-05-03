use chrono::{DateTime, Utc};
use compact_str::CompactString;
use serde::{de, Deserialize, Deserializer, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItemsResponse {
    pub items: Vec<PlaylistItem>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItem {
    pub snippet: Snippet,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    #[serde(deserialize_with = "published_at_timestamp")]
    pub published_at: i64,
    pub title: CompactString,
    pub resource_id: ResourceId,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceId {
    pub video_id: CompactString,
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
