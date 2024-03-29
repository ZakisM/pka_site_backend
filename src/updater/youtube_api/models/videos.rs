use compact_str::CompactString;
use serde::{de, Deserialize, Deserializer, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideosResponse {
    pub items: Vec<VideosItem>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideosItem {
    pub id: CompactString,
    pub snippet: Snippet,
    pub content_details: ContentDetails,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    pub title: CompactString,
    pub description: CompactString,
    pub published_at: CompactString,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentDetails {
    #[serde(deserialize_with = "duration_as_seconds")]
    pub duration: i32,
}

fn duration_as_seconds<'de, D>(deserializer: D) -> std::result::Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <CompactString>::deserialize(deserializer)?;
    let time = iso8601_duration::Duration::parse(&s).map_err(|e| {
        de::Error::custom(format!(
            "failed to parse iso8601_duration {} at {}",
            e.input, e.position
        ))
    })?;

    let time = time
        .to_chrono()
        .and_then(|t| t.num_seconds().try_into().ok())
        .ok_or_else(|| de::Error::custom(format!("failed to convert {} to chrono", time)))?;

    Ok(time)
}

#[cfg(test)]
mod tests {
    #[test]
    fn duration_to_seconds() {
        let s = "PT4M13S";
        let time = iso8601_duration::Duration::parse(s).expect("Failed to convert to iso8601");

        let time: i32 = time
            .to_chrono()
            .and_then(|t| t.num_seconds().try_into().ok())
            .expect("Failed to read time");

        assert_eq!(time, 253);
    }
}
