use std::sync::LazyLock;

use anyhow::Context;
use chrono::{NaiveTime, Timelike};
use compact_str::{CompactString, ToCompactString};
use regex::Regex;
use tracing::{error, info, warn};

use crate::conduit::sqlite::{pka_episode, pka_event, pka_youtube_details};
use crate::models::errors::ApiError;
use crate::models::pka_episode::PkaEpisode;
use crate::models::pka_event::PkaEvent;
use crate::models::pka_youtube_details::PkaYoutubeDetails;
use crate::updater::youtube_api::YoutubeApi;
use crate::{Repo, Result};

const WOODY_YOUTUBE_UPLOAD_PLAYLIST_ID: &str = "UUIPVJoHb_A5S3kcv3TJlyEg";

pub async fn get_latest_pka_episode_data(state: &Repo) -> Result<()> {
    info!("Checking playlist for missing episodes.");

    // Extract data from youtube_link
    let yt_api = YoutubeApi::new()?;

    let mut uploads = yt_api
        .get_latest_uploads(1, WOODY_YOUTUBE_UPLOAD_PLAYLIST_ID)
        .await?;

    if uploads.items.is_empty() {
        return Err(ApiError::new_internal_error("No playlist items found."));
    }

    // Sort uploads by publish date
    uploads
        .items
        .sort_by(|a, b| a.snippet.published_at.cmp(&b.snippet.published_at));

    let mut required_episode_number = pka_episode::latest(state).await?.floor() + 1.0;

    for upload in uploads.items.into_iter() {
        let required_episode = format!("PKA {}", required_episode_number).to_compact_string();

        info!("Looking for {}.", required_episode);

        if upload
            .snippet
            .title
            .to_lowercase()
            .contains(required_episode.to_lowercase().as_str())
        {
            info!(
                "Found {} in playlist. Attempting to extract video details.",
                required_episode
            );

            let details = yt_api
                .get_video_details(&upload.snippet.resource_id.video_id)
                .await?;

            let events = extract_pka_episode_events(
                required_episode_number,
                &details.snippet.description,
                &details.content_details.duration,
                &upload.snippet.published_at,
            )?;

            let pka_ep = PkaEpisode::new(
                required_episode_number,
                required_episode,
                upload.snippet.resource_id.video_id,
                upload.snippet.published_at,
            );

            let youtube_details = PkaYoutubeDetails::new(
                details.id,
                required_episode_number,
                details.snippet.title,
                details.content_details.duration,
            );

            if let Err(e) = pka_episode::insert(state, pka_ep).await {
                error!("Error adding latest pka_ep: {}", e);
            }

            for evt in events.into_iter() {
                if let Err(e) = pka_event::insert(state, evt).await {
                    error!("Error adding latest events: {}", e);
                }
            }

            if let Err(e) = pka_youtube_details::insert(state, youtube_details).await {
                error!("Error adding latest youtube_details: {}", e);
            }

            info!("Extracted video details.");

            required_episode_number += 1.0;
        } else {
            warn!("Could not find episode.");
        }
    }

    Ok(())
}

static TIMELINE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\d{1,2}(?::|;)\d{2}(?::|;)?\d*)(?:\s*-\s*)*\s*(.+)")
        .expect("Failed to create TIMELINE_REGEX.")
});

static UNPADDED_MINUTE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^(\d)(?::)"#).expect("Failed to create UNPADDED_MINUTE_REGEX"));

pub fn extract_pka_episode_events(
    ep_number: f32,
    data: &str,
    ep_length_seconds: &i32,
    upload_date: &i64,
) -> Result<Vec<PkaEvent>> {
    let mut events_without_duration: Vec<(CompactString, f32, i32, CompactString)> = Vec::new();

    for result in TIMELINE_REGEX.captures_iter(data) {
        let time_fragment = result
            .get(1)
            .map(|m| m.as_str())
            .context("Timeline regex missing timestamp capture")?;
        let timestamp = normalize_timestamp(time_fragment)?;

        let description = result
            .get(2)
            .map(|m| m.as_str())
            .context("Timeline regex missing description capture")
            .map(clean_description)?;

        let event_id = format!("{:<03}-{}", ep_number, timestamp).to_compact_string();

        events_without_duration.push((event_id, ep_number, timestamp, description));
    }

    if events_without_duration.is_empty() {
        return Err(ApiError::new_internal_error("Could not find any events"));
    }

    //sort events by timestamp
    events_without_duration.sort_by(|a, b| a.2.cmp(&b.2));

    let events = events_without_duration
        .iter()
        .enumerate()
        .map(|(index, event)| {
            let length_seconds = if let Some(next_event) = events_without_duration.get(index + 1) {
                next_event.2 - event.2
            } else {
                *ep_length_seconds - event.2
            }
            .max(1);

            PkaEvent::new(
                event.0.clone(),
                event.1,
                event.2,
                event.3.clone(),
                length_seconds,
                *upload_date,
            )
        })
        .collect();

    Ok(events)
}

fn normalize_timestamp(raw: &str) -> Result<i32> {
    let mut time_fragment = raw.replace(';', ":").trim_end_matches(':').to_owned();

    match time_fragment.matches(':').count() {
        1 => {
            if UNPADDED_MINUTE_REGEX.is_match(&time_fragment) {
                time_fragment = format!("0{time_fragment}");
            }
            time_fragment = format!("00:{time_fragment}");
        }
        2 => {
            if UNPADDED_MINUTE_REGEX.is_match(&time_fragment) {
                time_fragment = format!("0{time_fragment}");
            }
        }
        _ => return Err(ApiError::new_internal_error("Unknown timestamp found")),
    }

    let seconds = NaiveTime::parse_from_str(&time_fragment, "%H:%M:%S")
        .context("Failed to convert timeline fragment into timestamp")?
        .num_seconds_from_midnight() as i32;

    Ok(seconds)
}

fn clean_description(raw: &str) -> CompactString {
    raw.trim()
        .replace('’', "'")
        .replace("â€™", "'")
        .replace("â€¦", "…")
        .replace("â€“", "–")
        .replace("â€œ", "“")
        .replace("â€", "”")
        .to_compact_string()
}
