mod models;

use std::time::Duration;

use compact_str::ToCompactString;
use reqwest::Client;
use reqwest::ClientBuilder;
use strum_macros::Display;

use crate::models::errors::ApiError;
use crate::Result;
use crate::YT_API_KEY;

use self::models::playlist_items::PlaylistItemsResponse;
use self::models::videos::{VideosItem, VideosResponse};

pub struct YoutubeApi {
    client: Client,
}

// https://developers.google.com/youtube/v3/docs explains what these mean
#[derive(Debug, Display)]
#[strum(serialize_all = "camelCase")]
enum Part {
    ContentDetails,
    Snippet,
}

impl YoutubeApi {
    pub fn new() -> Result<Self> {
        let client = ClientBuilder::new()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(10))
            .gzip(true)
            .build()?;

        Ok(YoutubeApi { client })
    }

    pub async fn get_latest_uploads(
        &self,
        max_results: usize,
        playlist_id: &str,
    ) -> Result<PlaylistItemsResponse> {
        let parts = [Part::Snippet];

        let part = parts
            .iter()
            .map(|p| p.to_compact_string())
            .collect::<Vec<_>>()
            .join(",");

        let api_key = YT_API_KEY.get().await;

        let endpoint = format!(
            "https://www.googleapis.com/youtube/v3/playlistItems?part={}&maxResults={}&playlistId={}&key={}",
            part,
            max_results,
            playlist_id,
            api_key,
        );

        let res = self.client.get(&endpoint).send().await?;
        let data = serde_json::from_slice::<PlaylistItemsResponse>(&res.bytes().await?)?;

        Ok(data)
    }

    pub async fn get_video_details(&self, video_id: &str) -> Result<VideosItem> {
        let parts = [Part::ContentDetails, Part::Snippet];

        let part = parts
            .iter()
            .map(|p| p.to_compact_string())
            .collect::<Vec<_>>()
            .join(",");

        let api_key = YT_API_KEY.get().await;

        let endpoint = format!(
            "https://www.googleapis.com/youtube/v3/videos?part={}&id={}&key={}",
            part, video_id, api_key,
        );

        let res = self.client.get(&endpoint).send().await?;

        let full_data = serde_json::from_slice::<VideosResponse>(&res.bytes().await?)?;
        let data = full_data
            .items
            .first()
            .ok_or_else(|| ApiError::new_internal_error("Couldn't find video in item list."))?;

        Ok(data.to_owned())
    }
}
