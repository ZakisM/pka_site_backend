use chrono::{DateTime, NaiveTime, Timelike};
use regex::Regex;
use reqwest::{Client, StatusCode};

use crate::conduit::sqlite::{pka_episode, pka_event, pka_youtube_details};
use crate::models::errors::ApiError;
use crate::models::pka_episode::PkaEpisode;
use crate::models::pka_event::PkaEvent;
use crate::models::pka_youtube_details::PkaYoutubeDetails;
use crate::models::rss_feed::YoutubeRssFeed;
use crate::updater::youtube_api::YoutubeApi;
use crate::{Repo, Result};

const WOODY_YOUTUBE_RSS_FEED: &str =
    "https://www.youtube.com/feeds/videos.xml?channel_id=UCIPVJoHb_A5S3kcv3TJlyEg";

pub async fn get_latest_pka_episode_data(state: &Repo) -> Result<()> {
    let latest_episode_number = pka_episode::latest(state).await?.floor() + 1.0;
    let latest_episode = format!("PKA {}", latest_episode_number);

    info!("Looking for {}.", latest_episode);

    let client = Client::new();

    // Check RSS feed for latest youtube link
    let res = client.get(WOODY_YOUTUBE_RSS_FEED).send().await?;
    let data = String::from_utf8(res.bytes().await?.to_vec())?;

    let rss_feed_data: YoutubeRssFeed = quick_xml::de::from_str(&data)?;

    let latest_episode_lower_case = latest_episode.to_lowercase();

    let (youtube_link, uploaded) = rss_feed_data
        .entry()
        .iter()
        .find(|e| {
            e.title()
                .to_lowercase()
                .contains(&latest_episode_lower_case)
        })
        .map::<Result<_>, _>(|e| {
            let video_id = e.video_id().to_owned();
            let published = DateTime::parse_from_rfc3339(e.published())?;

            Ok((video_id, published.timestamp()))
        })
        .ok_or_else(|| ApiError::new("Couldn't find episode", StatusCode::NOT_FOUND))?
        .map_err(ApiError::from)?;

    // Extract data from youtube_link
    let yt_api = YoutubeApi::new();

    let details = yt_api.get_video_details(&youtube_link).await?;

    let events = extract_pka_episode_events(
        latest_episode_number,
        &details.snippet.description,
        &details.content_details.duration,
        &uploaded,
    )?;

    let pka_ep = PkaEpisode::new(
        latest_episode_number,
        latest_episode,
        youtube_link,
        uploaded,
    );

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

pub fn extract_pka_episode_events(
    ep_number: f32,
    data: &str,
    ep_length_seconds: &i32,
    upload_date: &i64,
) -> Result<Vec<PkaEvent>> {
    lazy_static! {
        static ref TIMELINE_REGEX: Regex = Regex::new(r"(\d{1,2}(?::|;)\d{2}(?::|;)?\d*)(?:\s*-\s*)*\s*(.+)")
            .expect("Failed to create TIMELINE_REGEX.");
        static ref UNPADDED_MINUTE_REGEX: Regex =
            Regex::new(r#"^(\d)(?::)"#).expect("Failed to create UNPADDED_MINUTE_REGEX");
    }

    let mut events_without_duration: Vec<(String, f32, i32, String)> = Vec::new();

    for result in TIMELINE_REGEX.captures_iter(data) {
        let mut time_date = result
            .get(1)
            .ok_or_else(|| ApiError::new_internal_error("Failed to get time_date from regex"))?
            .as_str()
            .replace(';', ":")
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
            .map_err(|_| ApiError::new_internal_error("Failed to convert time_date to timestamp"))?
            .num_seconds_from_midnight() as i32;

        let description = result
            .get(2)
            .ok_or_else(|| ApiError::new_internal_error("Failed to get description from regex"))?
            .as_str()
            .trim()
            .replace('’', "'")
            .replace("â€™", "'")
            .replace("â€¦", "…")
            .replace("â€“", "–")
            .replace("â€œ", "“")
            .replace("â€", "”")
            .to_owned();

        let event_id = format!("{:<03}-{}", ep_number, timestamp);

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
            //Calculate duration of each event
            let mut length_seconds =
                if let Some(next_event) = events_without_duration.get(index + 1) {
                    next_event.2 - event.2
                } else {
                    *ep_length_seconds - event.2
                };

            if length_seconds <= 0 {
                length_seconds = 1;
            }

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
