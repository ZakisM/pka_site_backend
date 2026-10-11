use std::sync::LazyLock;

use anyhow::{Context, bail};
use compact_str::{CompactString, ToCompactString};
use regex::{Regex, RegexBuilder};
use tracing::{error, info, warn};

use crate::Repo;
use crate::conduit::sqlite::{pka_episode, pka_event, pka_youtube_details};
use crate::models::pka_episode::PkaEpisode;
use crate::models::pka_event::PkaEvent;
use crate::models::pka_youtube_details::PkaYoutubeDetails;
use crate::updater::youtube_api::YoutubeApi;
use crate::updater::youtube_api::models::playlist_items::PlaylistItem;

const WOODY_YOUTUBE_UPLOAD_PLAYLIST_ID: &str = "UUIPVJoHb_A5S3kcv3TJlyEg";

static TITLE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"PKA\s*(?P<number>\d{3,})")
        .case_insensitive(true)
        .build()
        .expect("Failed to create TITLE_REGEX")
});

/// Adds any episodes uploaded since the newest one stored, oldest first, and
/// Says how many it added. Stops at the first it can't add (usually a video
/// Whose timeline isn't up yet), so the next check retries from there.
pub async fn load_new_episodes(repo: &Repo, yt_api: &YoutubeApi) -> anyhow::Result<usize> {
    info!("Checking playlist for missing episodes.");

    let stored_episode_number = pka_episode::latest(repo).await?.floor() as usize;

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

    let episodes_missing = latest_episode_number.saturating_sub(stored_episode_number);

    if episodes_missing == 0 {
        return Ok(0);
    }

    info!("Fetching {episodes_missing} missing episodes");

    let missing_uploads = yt_api
        .get_latest_uploads(episodes_missing, WOODY_YOUTUBE_UPLOAD_PLAYLIST_ID)
        .await?
        .items;
    let mut added = 0;

    for episode_number in (stored_episode_number + 1)..=latest_episode_number {
        let episode_name = format!("PKA {episode_number}").to_compact_string();

        let Some(matching_episode) = missing_uploads.iter().find(|ep| {
            TITLE_REGEX
                .captures(&ep.snippet.title)
                .and_then(|c| c.name("number"))
                .and_then(|n| n.as_str().parse::<usize>().ok())
                .is_some_and(|n| n == episode_number)
        }) else {
            warn!("Could not find {episode_name} in playlist.");
            continue;
        };

        info!("Found {episode_name} in playlist. Attempting to extract video details.");

        if let Err(e) = extract_then_save_events(
            repo,
            yt_api,
            episode_name.clone(),
            episode_number as f32,
            matching_episode,
        )
        .await
        {
            error!("Couldn't add {episode_name}: {e:#}");
            break;
        }

        added += 1;
    }

    Ok(added)
}

async fn extract_then_save_events(
    repo: &Repo,
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

    save_episode(repo, pka_ep, youtube_details, events).await?;

    info!("Extracted successfully.");

    Ok(())
}

/// Stores an episode with its video and moments, all or nothing: a half-saved
/// Episode would be listed but unplayable, and never retried.
pub async fn save_episode(
    repo: &Repo,
    episode: PkaEpisode,
    youtube_details: PkaYoutubeDetails,
    events: Vec<PkaEvent>,
) -> anyhow::Result<()> {
    let number = episode.number;
    let mut tx = repo.begin().await?;

    pka_episode::insert(&mut *tx, episode)
        .await
        .with_context(|| format!("Failed to save episode {number}"))?;

    for event in events {
        let event_id = event.event_id();

        pka_event::insert(&mut *tx, event)
            .await
            .with_context(|| format!("Failed to save moment '{event_id}'"))?;
    }

    pka_youtube_details::insert(&mut *tx, youtube_details)
        .await
        .with_context(|| format!("Failed to save the video for episode {number}"))?;

    tx.commit().await?;

    Ok(())
}

static TIMELINE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\d{1,2}(?::|;)\d{2}(?::|;)?\d*)(?:\s*-\s*)*\s*(.+)")
        .expect("Failed to create TIMELINE_REGEX")
});

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

    // In time order, and one moment per timestamp: the timestamp is part of
    // The moment's ID, and a repeated line in a description would collide.
    events_without_duration.sort_by_key(|a| a.2);
    events_without_duration.dedup_by_key(|a| a.2);

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
    let time_fragment = raw.replace(';', ":").trim_end_matches(':').to_owned();

    let parts: Vec<&str> = time_fragment.split(':').collect();

    let seconds = match parts.len() {
        1 => parts[0].parse::<i32>().with_context(|| {
            format!("Failed to parse seconds from '{time_fragment}' (derived from '{raw}')")
        })?,
        2 => {
            let minutes = parts[0].parse::<i32>().with_context(|| {
                format!("Failed to parse minutes from '{time_fragment}' (derived from '{raw}')")
            })?;
            let seconds = parts[1].parse::<i32>().with_context(|| {
                format!("Failed to parse seconds from '{time_fragment}' (derived from '{raw}')")
            })?;
            minutes * 60 + seconds
        }
        3 => {
            let hours = parts[0].parse::<i32>().with_context(|| {
                format!("Failed to parse hours from '{time_fragment}' (derived from '{raw}')")
            })?;
            let minutes = parts[1].parse::<i32>().with_context(|| {
                format!("Failed to parse minutes from '{time_fragment}' (derived from '{raw}')")
            })?;
            let seconds = parts[2].parse::<i32>().with_context(|| {
                format!("Failed to parse seconds from '{time_fragment}' (derived from '{raw}')")
            })?;
            hours * 3600 + minutes * 60 + seconds
        }
        _ => bail!("Unknown timestamp format: '{time_fragment}' (derived from '{raw}')"),
    };

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_timestamp() {
        // Standard formats
        assert_eq!(normalize_timestamp("12:34").unwrap(), 754);
        assert_eq!(normalize_timestamp("2:34").unwrap(), 154);
        assert_eq!(normalize_timestamp("1:02:03").unwrap(), 3723);

        // Out-of-range bounds
        assert_eq!(normalize_timestamp("2:36:66").unwrap(), 9426); // Out-of-range seconds!
        assert_eq!(normalize_timestamp("02:36:66").unwrap(), 9426);
        assert_eq!(normalize_timestamp("2:80:05").unwrap(), 12005); // Out-of-range minutes!

        // Alternative separators (semicolons instead of colons)
        assert_eq!(normalize_timestamp("12;34").unwrap(), 754);
        assert_eq!(normalize_timestamp("1;02;03").unwrap(), 3723);

        // Trailing separator trimming
        assert_eq!(normalize_timestamp("12:34:").unwrap(), 754);
        assert_eq!(normalize_timestamp("1:02:03:").unwrap(), 3723);

        // Single part (seconds only)
        assert_eq!(normalize_timestamp("45").unwrap(), 45);

        // Error cases
        assert!(normalize_timestamp("12:34:56:78").is_err()); // Too many parts
        assert!(normalize_timestamp("abc").is_err()); // Non-digit characters
        assert!(normalize_timestamp("12:ab").is_err()); // Non-digit parts
    }

    #[test]
    fn test_extract_keeps_one_moment_per_timestamp() {
        let events = extract_pka_episode_events(
            900.0,
            "0:00 Intro\n12:34 Kyle's bike\n12:34 Kyle's bike (again)\n1:02:03 Outro",
            &4000,
            &0,
        )
        .unwrap();

        assert_eq!(
            events.iter().map(PkaEvent::timestamp).collect::<Vec<_>>(),
            vec![0, 754, 3723]
        );
        assert_eq!(events[1].description(), "Kyle's bike");
        assert_eq!(
            events
                .iter()
                .map(PkaEvent::length_seconds)
                .collect::<Vec<_>>(),
            vec![754, 2969, 277]
        );
    }
}
