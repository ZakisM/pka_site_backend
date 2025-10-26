use std::sync::LazyLock;

use anyhow::{bail, Context};
use chrono::{NaiveTime, Timelike};
use compact_str::{CompactString, ToCompactString};
use regex::{Regex, RegexBuilder};
use tracing::{error, info, warn};

use crate::conduit::sqlite::{pka_episode, pka_event, pka_youtube_details};
use crate::models::pka_episode::PkaEpisode;
use crate::models::pka_event::PkaEvent;
use crate::models::pka_youtube_details::PkaYoutubeDetails;
use crate::updater::youtube_api::models::playlist_items::PlaylistItem;
use crate::updater::youtube_api::YoutubeApi;
use crate::Repo;

const WOODY_YOUTUBE_UPLOAD_PLAYLIST_ID: &str = "UUIPVJoHb_A5S3kcv3TJlyEg";

static TITLE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"PKA (?P<number>\d{3,})")
        .case_insensitive(true)
        .build()
        .expect("Failed to create TITLE_REGEX")
});

pub async fn load_new_episodes(state: &Repo) -> anyhow::Result<()> {
    info!("Checking playlist for missing episodes.");

    let stored_episode_number = pka_episode::latest(state).await?.floor() as usize;

    let yt_api = YoutubeApi::new()?;

    let latest_episode_title = yt_api
        .get_latest_uploads(1, WOODY_YOUTUBE_UPLOAD_PLAYLIST_ID)
        .await?
        .items
        .into_iter()
        .map(|item| item.snippet.title)
        .next()
        .context("Failed to find latest episode title")?;

    let latest_episode_number = TITLE_REGEX
        .captures(&latest_episode_title)
        .and_then(|capture| capture.name("number"))
        .context("Failed to find latest episode number from title")?
        .as_str()
        .parse::<f32>()
        .context("Failed to parse latest episode number to f32")?
        as usize;

    let episodes_missing = (latest_episode_number).saturating_sub(stored_episode_number);

    if episodes_missing > 0 {
        info!("Fetching {episodes_missing} missing episodes");

        let missing_uploads = yt_api
            .get_latest_uploads(episodes_missing, WOODY_YOUTUBE_UPLOAD_PLAYLIST_ID)
            .await?
            .items;

        for episode_number in (stored_episode_number + 1)..=latest_episode_number {
            let episode_name = format!("PKA {}", episode_number).to_compact_string();

            match missing_uploads.iter().find(|ep| {
                ep.snippet
                    .title
                    .to_lowercase()
                    .contains(episode_name.to_lowercase().as_str())
            }) {
                Some(matching_episode) => {
                    info!("Found {episode_name} in playlist. Attempting to extract video details.");

                    extract_then_save_events(
                        state,
                        &yt_api,
                        episode_name,
                        episode_number as f32,
                        matching_episode,
                    )
                    .await?;
                }
                None => {
                    warn!("Could not find {episode_name} in playlist.");
                }
            }
        }
    }

    Ok(())
}

async fn extract_then_save_events(
    state: &Repo,
    yt_api: &YoutubeApi,
    name: CompactString,
    number: f32,
    playlist_item: &PlaylistItem,
) -> anyhow::Result<()> {
    let details = yt_api
        .get_video_details(&playlist_item.snippet.resource_id.video_id)
        .await?;

    let events = extract_pka_episode_events(
        number,
        &details.snippet.description,
        &details.content_details.duration,
        &playlist_item.snippet.published_at,
    )?;

    let pka_ep = PkaEpisode::new(
        number,
        name,
        playlist_item.snippet.resource_id.video_id.to_owned(),
        playlist_item.snippet.published_at,
    );

    let youtube_details = PkaYoutubeDetails::new(
        details.id,
        number,
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

    info!("Extracted successfully.");

    Ok(())
}

static TIMELINE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\d{1,2}(?::|;)\d{2}(?::|;)?\d*)(?:\s*-\s*)*\s*(.+)")
        .expect("Failed to create TIMELINE_REGEX")
});

static UNPADDED_MINUTE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^(\d)(?::)"#).expect("Failed to create UNPADDED_MINUTE_REGEX"));

pub fn extract_pka_episode_events(
    ep_number: f32,
    data: &str,
    ep_length_seconds: &i32,
    upload_date: &i64,
) -> anyhow::Result<Vec<PkaEvent>> {
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
        bail!("Could not find any events");
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

fn normalize_timestamp(raw: &str) -> anyhow::Result<i32> {
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
        _ => bail!("Unknown timestamp found"),
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
