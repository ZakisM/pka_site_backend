use std::fmt;
use std::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoDetailsRoot {
    #[serde(with = "serde_with::json::nested")]
    pub video_details: VideoDetails,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoDetails {
    pub video_id: String,
    pub title: String,
    #[serde(deserialize_with = "any_from_str")]
    pub length_seconds: i16,
    pub short_description: String,
    pub average_rating: f32,
    #[serde(deserialize_with = "any_from_str")]
    pub view_count: i32,
    pub author: String,
}

fn any_from_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as std::str::FromStr>::Err: fmt::Display,
{
    let s = <String>::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EpisodesFileRoot {
    pub episodes: Vec<EpisodeFromFile>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct EpisodeFromFile {
    pub identifier: String,
    pub number: f32,
    pub date_time: String,
    date: String,
    pub you_tube: String,
    timelined: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PkaInfoRoot {
    #[serde(rename = "Timeline")]
    pub timeline: PkaInfoTimeline,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PkaInfoTimeline {
    #[serde(rename = "Timestamps")]
    pub timestamps: Vec<PkaInfoTimestamp>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PkaInfoTimestamp {
    #[serde(rename = "HMS")]
    pub hms: String,
    #[serde(rename = "Value")]
    pub value: String,
}
