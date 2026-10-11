pub mod models;

use std::time::Duration;

use anyhow::Context;
use reqwest::{Client, ClientBuilder};
use serde::de::DeserializeOwned;

use self::models::playlist_items::PlaylistItemsResponse;
use self::models::videos::{VideosItem, VideosResponse};

const API: &str = "https://www.googleapis.com/youtube/v3";

/// The two YouTube Data API calls the updater makes. One client, so checks
/// Reuse its connections.
pub struct YoutubeApi {
    client: Client,
    api_key: String,
}

impl YoutubeApi {
    pub fn new(api_key: String) -> anyhow::Result<Self> {
        let client = ClientBuilder::new()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(10))
            .gzip(true)
            .build()?;

        Ok(YoutubeApi { client, api_key })
    }

    // https://developers.google.com/youtube/v3/docs explains the parameters.
    async fn get<T: DeserializeOwned>(
        &self,
        resource: &str,
        query: &[(&str, &str)],
    ) -> anyhow::Result<T> {
        // The key travels in the URL, and reqwest's errors print the URL:
        // `without_url` keeps it out of the logs.
        let response = self
            .client
            .get(format!("{API}/{resource}"))
            .query(query)
            .query(&[("key", &self.api_key)])
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(reqwest::Error::without_url)
            .with_context(|| format!("YouTube {resource} request failed"))?;
        let body = response
            .bytes()
            .await
            .map_err(reqwest::Error::without_url)?;

        serde_json::from_slice(&body)
            .with_context(|| format!("Unexpected YouTube {resource} response"))
    }

    pub async fn get_latest_uploads(
        &self,
        max_results: usize,
        playlist_id: &str,
    ) -> anyhow::Result<PlaylistItemsResponse> {
        self.get(
            "playlistItems",
            &[
                ("part", "snippet"),
                ("maxResults", &max_results.to_string()),
                ("playlistId", playlist_id),
            ],
        )
        .await
    }

    pub async fn get_video_details(&self, video_id: &str) -> anyhow::Result<VideosItem> {
        let response: VideosResponse = self
            .get(
                "videos",
                &[("part", "contentDetails,snippet"), ("id", video_id)],
            )
            .await?;

        response
            .items
            .into_iter()
            .next()
            .with_context(|| format!("YouTube has no video {video_id}"))
    }
}
