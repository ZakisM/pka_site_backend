use std::sync::Arc;

use chrono::{NaiveTime, Timelike};
use regex::Regex;
use reqwest::{Client, StatusCode};
use tokio::time;
use tokio::time::Duration;

use crate::conduit::{pka_episode, pka_event, pka_youtube_details};
use crate::models::errors::ApiError;
use crate::models::pka_episode::PkaEpisode;
use crate::models::pka_event::PkaEvent;
use crate::models::pka_youtube_details::PkaYoutubeDetails;
use crate::updater::youtube_api::YoutubeAPI;
use crate::YT_API_KEY;
use crate::{Repo, Result};

pub const PKA_DESCRIPTIONS_FOLDER: &str = "PKA-Descriptions";

const WOODY_YOUTUBE_RSA_FEED: &str =
    "https://www.youtube.com/feeds/videos.xml?channel_id=UCIPVJoHb_A5S3kcv3TJlyEg";

pub async fn spawn_get_latest_worker(state: Arc<Repo>) {
    loop {
        info!("Checking for latest episode...");

        if let Err(e) = get_latest_pka_episode_data(&state).await {
            error!("get_latest_worker error: {}", e);
        } else {
            info!("Successfully added latest episode");
        }

        //Check once every 24 hours.
        time::delay_for(Duration::from_secs(86400)).await;
    }
}

pub async fn get_latest_pka_episode_data(state: &Repo) -> Result<()> {
    let latest_episode_number = pka_episode::latest(state).await? + 1.0;
    let latest_episode = format!("PKA {}", latest_episode_number);

    info!("Looking for {}.", latest_episode);

    let client = Client::new();

    // Check RSA feed for latest youtube link
    let res = client.get(WOODY_YOUTUBE_RSA_FEED).send().await?;
    let data = String::from_utf8(res.bytes().await?.to_vec())?;

    let c = feed_rs::parser::parse(data.as_bytes())?;

    let (youtube_link, uploaded) = c
        .entries
        .into_iter()
        .filter(|e| e.title.is_some() && e.published.is_some())
        .find(|e| e.title.as_ref().unwrap().content.contains(&latest_episode))
        .map(|e| {
            (
                e.id.replace("yt:video:", ""),
                e.published.unwrap().timestamp(),
            )
        })
        .ok_or_else(|| ApiError::new("Couldn't find episode", StatusCode::NOT_FOUND))?;

    // Extract data from youtube_link
    let yt_api = YoutubeAPI::new(&YT_API_KEY);

    let details = yt_api.get_video_details(&youtube_link).await?;

    let pka_ep = PkaEpisode::new(
        latest_episode_number,
        latest_episode,
        youtube_link,
        uploaded,
    );

    let events = extract_pka_episode_events(latest_episode_number, &details.snippet.description)?;

    let youtube_details = PkaYoutubeDetails::new(
        details.id,
        latest_episode_number,
        details.snippet.title,
        details.content_details.duration,
    );

    info!("Extracted video details.");

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

    Ok(())
}

pub fn extract_pka_episode_events(ep_number: f32, data: &str) -> Result<Vec<PkaEvent>> {
    lazy_static! {
        static ref TIMELINE_REGEX: Regex = Regex::new(r"(\d{1,2}:\d{2}:?\d*)(?:\s*-\s*)*\s*(.+)")
            .expect("Failed to create TIMELINE_REGEX.");
        static ref UNPADDED_MINUTE_REGEX: Regex =
            Regex::new(r#"^(\d)(?::)"#).expect("Failed to create UNPADDED_MINUTE_REGEX");
    }

    let mut events = Vec::new();

    for result in TIMELINE_REGEX.captures_iter(data) {
        let mut time_date = result
            .get(1)
            .ok_or_else(|| ApiError::new_internal_error("Failed to get time_date from regex"))?
            .as_str()
            .trim_end_matches(':')
            .to_owned();

        //must ensure format is 00:00:00 ie H:M:S
        match time_date.matches(':').count() {
            1 => {
                // if minute doesn't have a padding 0 then must add it.
                if UNPADDED_MINUTE_REGEX.is_match(&time_date) {
                    time_date = format!("0{}", time_date);
                }
                //then add leading 00:
                time_date = format!("00:{}", time_date);
            }
            2 => {
                if UNPADDED_MINUTE_REGEX.is_match(&time_date) {
                    time_date = format!("0{}", time_date);
                }
            }
            _ => return Err(ApiError::new_internal_error("Unknown timestamp found")),
        };

        let timestamp = NaiveTime::parse_from_str(&time_date, "%H:%M:%S")
            .or_else(|_| {
                Err(ApiError::new_internal_error(
                    "Failed to convert time_date to timestamp",
                ))
            })?
            .num_seconds_from_midnight() as i64;

        let description = result
            .get(2)
            .ok_or_else(|| ApiError::new_internal_error("Failed to get description from regex"))?
            .as_str()
            .trim()
            .replace("’", "'")
            .replace("â€™", "'")
            .replace("â€¦", "…")
            .replace("â€“", "–")
            .replace("â€œ", "“")
            .replace("â€", "”")
            .to_owned();

        let event_id = format!("{:<03}-{}", ep_number, timestamp);

        let event = PkaEvent::new(event_id, ep_number, timestamp, description);
        events.push(event);
    }

    Ok(events)
}
