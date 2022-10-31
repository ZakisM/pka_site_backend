use serde::{de, Deserialize, Deserializer, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideosResponse {
    pub items: Vec<VideosItem>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideosItem {
    pub id: String,
    pub snippet: Snippet,
    pub content_details: ContentDetails,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    pub title: String,
    pub description: String,
    pub published_at: String,
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
    let s = <String>::deserialize(deserializer)?;
    let time = iso8601_duration::Duration::parse(&s)
        .map_err(de::Error::custom)?
        .to_std();
    let time = chrono::Duration::from_std(time)
        .map_err(de::Error::custom)?
        .num_seconds() as i32;

    Ok(time)
}

#[cfg(test)]
mod tests {
    #[test]
    fn duration_to_seconds() {
        let s = "PT4M13S";
        let time = iso8601_duration::Duration::parse(s)
            .expect("Failed to convert to iso8601")
            .to_std();
        let time = chrono::Duration::from_std(time)
            .expect("Failed to read time")
            .num_seconds() as i32;

        assert_eq!(time, 253);
    }
}
