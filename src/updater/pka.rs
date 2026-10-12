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

/// What a check for new episodes found.
#[derive(Debug, Default)]
pub struct NewEpisodes {
    pub added: usize,
    /// The next episode is up, but its description has no usable timeline
    /// Yet.
    pub awaiting_timeline: bool,
}

/// Why an episode couldn't be added.
enum NotAdded {
    // Nothing in its description reads as a timeline: usually because it
    // Isn't up yet, and retrying soon won't change that either way.
    NoTimeline(anyhow::Error),
    Failed(anyhow::Error),
}

/// Adds any episodes uploaded since the newest one stored, oldest first.
/// Stops at the first it can't add (usually a video whose timeline isn't up
/// Yet), so the next check retries from there.
pub async fn load_new_episodes(repo: &Repo, yt_api: &YoutubeApi) -> anyhow::Result<NewEpisodes> {
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
        return Ok(NewEpisodes::default());
    }

    info!("Fetching {episodes_missing} missing episodes");

    let missing_uploads = yt_api
        .get_latest_uploads(episodes_missing, WOODY_YOUTUBE_UPLOAD_PLAYLIST_ID)
        .await?
        .items;
    let mut found = NewEpisodes::default();

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

        match extract_then_save_events(
            repo,
            yt_api,
            episode_name.clone(),
            episode_number as f32,
            matching_episode,
        )
        .await
        {
            Ok(()) => found.added += 1,
            Err(NotAdded::NoTimeline(e)) => {
                warn!("{episode_name} has no usable timeline yet: {e:#}");
                found.awaiting_timeline = true;
                break;
            }
            Err(NotAdded::Failed(e)) => {
                error!("Couldn't add {episode_name}: {e:#}");
                break;
            }
        }
    }

    Ok(found)
}

async fn extract_then_save_events(
    repo: &Repo,
    yt_api: &YoutubeApi,
    name: CompactString,
    number: f32,
    playlist_item: &PlaylistItem,
) -> Result<(), NotAdded> {
    let details = yt_api
        .get_video_details(&playlist_item.snippet.resource_id.video_id)
        .await
        .map_err(NotAdded::Failed)?;

    let events = extract_pka_episode_events(
        number,
        &details.snippet.description,
        &details.content_details.duration,
        &playlist_item.snippet.published_at,
    )
    .map_err(NotAdded::NoTimeline)?;

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

    save_episode(repo, pka_ep, youtube_details, events)
        .await
        .map_err(NotAdded::Failed)?;

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

// A timestamp partway through a line, which may start another moment:
// "1:13:54 Next trip ideas (hosts disagree) 1:19:07 Woody has bug bites".
static INNER_TIMESTAMP_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|\s)(\d{1,2}[:;]\d{2}(?:[:;]\d{2})?)\s+\S")
        .expect("Failed to create INNER_TIMESTAMP_REGEX")
});

static LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:https?://|www\.)\S+").expect("Failed to create LINK_REGEX")
});

// Words that put a time inside a sentence ("at 12:00", "around 2:30"), where
// It's part of the moment rather than the start of another.
const TIME_IN_PROSE: [&str; 9] = [
    "at", "around", "about", "by", "from", "to", "until", "till", "@",
];

fn timestamp_parts(timestamp: &str) -> usize {
    timestamp.split([':', ';']).count()
}

/// One captured line, split wherever another moment starts partway through
/// It: a later time in the line's own format, not one inside a sentence.
fn split_line(timestamp: &str, text: &str) -> anyhow::Result<Vec<(i32, String)>> {
    let mut pieces = vec![(normalize_timestamp(timestamp)?, String::new())];
    let mut rest = text;

    while let Some(found) = INNER_TIMESTAMP_REGEX
        .captures_iter(rest)
        .filter_map(|capture| capture.get(1))
        .find(|candidate| {
            let before = rest[..candidate.start()].trim_end();
            let in_prose = TIME_IN_PROSE.iter().any(|word| {
                before
                    .rsplit(char::is_whitespace)
                    .next()
                    .is_some_and(|last| last.eq_ignore_ascii_case(word))
            });
            let later = normalize_timestamp(candidate.as_str()).is_ok_and(|seconds| {
                pieces
                    .last()
                    .is_some_and(|(previous, _)| seconds > *previous)
            });

            !before.is_empty()
                && !in_prose
                && later
                && timestamp_parts(candidate.as_str()) == timestamp_parts(timestamp)
        })
    {
        if let Some((_, current)) = pieces.last_mut() {
            current.push_str(&rest[..found.start()]);
        }

        pieces.push((normalize_timestamp(found.as_str())?, String::new()));
        rest = &rest[found.end()..];
    }

    if let Some((_, current)) = pieces.last_mut() {
        current.push_str(rest);
    }

    Ok(pieces)
}

/// The moments in a video description, in time order, one per timestamp.
/// Moments that point past the end of the video, or that are nothing but a
/// Link, are left out: they can't be played or searched. A description with
/// None left has no timeline.
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
        let text = result
            .get(2)
            .map(|m| m.as_str())
            .context("Timeline regex missing description capture")?;

        for (timestamp, raw_description) in split_line(time_fragment, text)? {
            let description = clean_description(&raw_description);
            let past_the_end = *ep_length_seconds > 0 && timestamp >= *ep_length_seconds;

            if description.is_empty() || past_the_end {
                continue;
            }

            let event_id = format!("{:<03}-{}", ep_number, timestamp).to_compact_string();

            events_without_duration.push((event_id, ep_number, timestamp, description));
        }
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

// Characters that take no space: a byte-order mark or a zero-width joiner in
// "Outro" would stop it matching a search for "outro".
const INVISIBLE: [char; 6] = [
    '\u{feff}', '\u{200b}', '\u{200c}', '\u{200d}', '\u{2060}', '\u{00ad}',
];

// What a removed link leaves at the end: "Woody's vlogs: https://…".
const DANGLING: [char; 7] = [':', '-', '–', '—', '|', ',', ';'];

fn clean_description(raw: &str) -> CompactString {
    let fixed = raw
        .replace('’', "'")
        .replace("â€™", "'")
        .replace("â€¦", "…")
        .replace("â€“", "–")
        .replace("â€œ", "“")
        .replace("â€", "”")
        .replace(INVISIBLE, "");
    let without_links = LINK_REGEX.replace_all(&fixed, "");

    without_links
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_matches(|c: char| c.is_whitespace() || DANGLING.contains(&c))
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

    fn timeline(description: &str, length_seconds: i32) -> Vec<(i32, String)> {
        extract_pka_episode_events(246.0, description, &length_seconds, &0)
            .map(|events| {
                events
                    .iter()
                    .map(|event| (event.timestamp(), event.description().to_owned()))
                    .collect()
            })
            .unwrap_or_default()
    }

    #[test]
    fn test_splits_several_moments_on_one_line() {
        assert_eq!(
            timeline(
                "1:13:54 Next trip ideas (hosts disagree) 1:19:07 Woody has a ton of bug bites 1:19:47 Chicken Updates",
                14_400
            ),
            vec![
                (4434, "Next trip ideas (hosts disagree)".to_owned()),
                (4747, "Woody has a ton of bug bites".to_owned()),
                (4787, "Chicken Updates".to_owned()),
            ]
        );
    }

    #[test]
    fn test_leaves_times_inside_a_sentence_alone() {
        assert_eq!(
            timeline(
                "1:13:54 Kyle's 2:30 mile time\n12:00 Lunch at 13:30 with Woody\n20:00 Ice wakes at 22:15 again",
                14_400
            ),
            vec![
                (720, "Lunch at 13:30 with Woody".to_owned()),
                (1200, "Ice wakes at 22:15 again".to_owned()),
                (4434, "Kyle's 2:30 mile time".to_owned()),
            ]
        );
    }

    #[test]
    fn test_removes_invisible_characters_and_links() {
        assert_eq!(
            timeline(
                "4:15:18 Outro\u{feff}\n2:55:25 Woody's Day in The Life Vlogs: https://www.youtube.com/watch?v=UlTcDI5XPcA",
                16_000
            ),
            vec![
                (10_525, "Woody's Day in The Life Vlogs".to_owned()),
                (15_318, "Outro".to_owned()),
            ]
        );
    }

    #[test]
    fn test_drops_link_only_moments_and_moments_past_the_end() {
        assert_eq!(
            timeline(
                "0:09:33 http://www.worldstarhiphop.com/videos/video.php?v=abc\n1:00:00 Real moment\n3:59:05 Taylor asks Kat about board games",
                3 * 3600 + 53 * 60
            ),
            vec![(3600, "Real moment".to_owned())]
        );
    }

    #[test]
    fn test_a_description_with_nothing_usable_has_no_timeline() {
        assert!(
            extract_pka_episode_events(134.0, "0:09:33 http://www.example.com/video", &10_000, &0)
                .is_err()
        );
        assert!(extract_pka_episode_events(7.0, "No timestamps here at all", &10_000, &0).is_err());
    }
}
