use std::fmt;
use std::fmt::Formatter;

use reqwest::Client;
use serde::{de, Deserialize, Deserializer, Serialize};

use crate::models::errors::ApiError;
use crate::Result;

pub struct YoutubeAPI<'a> {
    key: &'a str,
    client: Client,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeAPIResponse {
    pub items: Vec<YoutubeAPIData>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeAPIData {
    pub id: String,
    pub snippet: Snippet,
    pub content_details: ContentDetails,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    pub title: String,
    pub description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentDetails {
    #[serde(deserialize_with = "duration_as_seconds")]
    pub duration: i16,
}

fn duration_as_seconds<'de, D>(deserializer: D) -> std::result::Result<i16, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <String>::deserialize(deserializer)?;
    let time = iso8601_duration::Duration::parse(&s)
        .map_err(de::Error::custom)?
        .to_std();
    let time = chrono::Duration::from_std(time)
        .map_err(de::Error::custom)?
        .num_seconds() as i16;

    Ok(time)
}

// https://developers.google.com/youtube/v3/docs explains what these mean
#[derive(Debug)]
enum Part {
    ContentDetails,
    Snippet,
}

impl fmt::Display for Part {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> YoutubeAPI<'a> {
    pub fn new(key: &'a str) -> Self {
        let client = Client::new();

        YoutubeAPI { key, client }
    }

    pub async fn get_video_details(&self, video_id: &str) -> Result<YoutubeAPIData> {
        let parts = vec![Part::ContentDetails, Part::Snippet];

        let part = parts
            .into_iter()
            .map(|p| {
                let p = p.to_string();
                p[..1].to_lowercase() + &p[1..]
            })
            .collect::<Vec<String>>()
            .join(r#","#);

        let endpoint = format!(
            "https://www.googleapis.com/youtube/v3/videos?part={}&id={}&key={}",
            part, video_id, self.key
        );

        let res = self.client.get(&endpoint).send().await?;

        let full_data = serde_json::from_slice::<YoutubeAPIResponse>(&res.bytes().await?)?;
        let data = full_data
            .items
            .get(0)
            .ok_or_else(|| ApiError::new_internal_error("Couldn't find video in item list."))?;

        Ok(data.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_to_seconds() {
        let s = "PT4M13S";
        let time = iso8601_duration::Duration::parse(&s)
            .expect("Failed to convert to iso8601")
            .to_std();
        let time = chrono::Duration::from_std(time)
            .expect("Failed to read time")
            .num_seconds() as i16;

        assert_eq!(time, 253);
    }
}
