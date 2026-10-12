//! Keeping stored episodes in step with YouTube, as its API terms require of
//! data fetched from it: nothing kept unchecked for more than 30 days, and
//! what's shown consistent with what YouTube has now.
//!
//! Titles and lengths always follow YouTube. Moments follow the video's
//! description when it has a timeline (minus moments it can't play, see
//! `extract_pka_episode_events`); when it has none, the episode's moments came
//! from elsewhere and are kept, as are locked episodes' (see the migration).
//! A video gone from YouTube hides its episode, which is deleted once the
//! video has been missing for a week.

use std::collections::HashMap;

use anyhow::Context;
use compact_str::CompactString;
use tracing::info;

use crate::Repo;
use crate::catalog::episode_key;
use crate::conduit::sqlite::{pka_episode, pka_event, pka_youtube_details};
use crate::models::pka_episode::SyncState;
use crate::models::pka_event::PkaEvent;
use crate::updater::pka::extract_pka_episode_events;
use crate::updater::youtube_api::YoutubeApi;

// Long enough to ride out a video made private for an edit, or a glitch on
// YouTube's side, and well inside the 30 days.
const DELETE_AFTER_MISSING_SECONDS: i64 = 7 * 24 * 60 * 60;

/// What YouTube has now for one video.
#[derive(Clone, Debug)]
pub struct Video {
    pub title: CompactString,
    pub length_seconds: i32,
    pub description: CompactString,
}

#[derive(Debug, PartialEq)]
pub enum Change {
    Details {
        number: f32,
        title: CompactString,
        length_seconds: i32,
    },
    Moments {
        number: f32,
        moments: Vec<PkaEvent>,
    },
    MissingSince {
        number: f32,
        since: i64,
    },
    Found {
        number: f32,
    },
    Delete {
        number: f32,
    },
}

// Moments compared by what's shown: when, what, and for how long.
fn same_moments(a: &[PkaEvent], b: &[PkaEvent]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b).all(|(a, b)| {
            a.timestamp() == b.timestamp()
                && a.description() == b.description()
                && a.length_seconds() == b.length_seconds()
        })
}

// The site's own moments: locked by hand, or a part added by hand.
fn moments_locked(episode: &SyncState) -> bool {
    episode.moments_locked || episode.number.fract() != 0.0
}

fn plan_episode(
    episode: &SyncState,
    stored: &[PkaEvent],
    video: Option<&Video>,
    now: i64,
) -> Vec<Change> {
    let number = episode.number;
    let Some(video) = video else {
        return match episode.missing_since {
            None => vec![Change::MissingSince { number, since: now }],
            Some(since) if now - since >= DELETE_AFTER_MISSING_SECONDS => {
                vec![Change::Delete { number }]
            }
            Some(_) => Vec::new(),
        };
    };
    let mut changes = Vec::new();

    if episode.missing_since.is_some() {
        changes.push(Change::Found { number });
    }

    if video.title != episode.title || video.length_seconds != episode.length_seconds {
        changes.push(Change::Details {
            number,
            title: video.title.clone(),
            length_seconds: video.length_seconds,
        });
    }

    if moments_locked(episode) {
        return changes;
    }

    // No timeline in the description: the episode's moments are from
    // elsewhere, and stay.
    if let Ok(moments) = extract_pka_episode_events(
        number,
        &video.description,
        &video.length_seconds,
        &episode.upload_date,
    ) && !same_moments(&moments, stored)
    {
        changes.push(Change::Moments { number, moments });
    }

    changes
}

/// Every change needed to bring the stored episodes in line with `videos`
/// (YouTube's answer, by video id). Writes nothing.
pub fn plan(
    episodes: &[SyncState],
    moments: &HashMap<i64, Vec<PkaEvent>>,
    videos: &HashMap<CompactString, Video>,
    now: i64,
) -> Vec<Change> {
    episodes
        .iter()
        .flat_map(|episode| {
            let stored = moments
                .get(&episode_key(episode.number))
                .map_or(&[][..], Vec::as_slice);

            plan_episode(episode, stored, videos.get(&episode.video_id), now)
        })
        .collect()
}

/// Applies `changes` in one transaction: all of them or none.
pub async fn apply(repo: &Repo, changes: Vec<Change>) -> anyhow::Result<()> {
    let mut tx = repo.begin().await?;

    for change in changes {
        match change {
            Change::Details {
                number,
                title,
                length_seconds,
            } => {
                pka_youtube_details::update_title_and_length(
                    &mut *tx,
                    number,
                    &title,
                    length_seconds,
                )
                .await?;
            }
            Change::Moments { number, moments } => {
                pka_event::delete_for_episode(&mut *tx, number).await?;

                for moment in moments {
                    pka_event::insert(&mut *tx, moment).await?;
                }
            }
            Change::MissingSince { number, since } => {
                pka_episode::set_missing_since(&mut *tx, number, Some(since)).await?;
            }
            Change::Found { number } => {
                pka_episode::set_missing_since(&mut *tx, number, None).await?;
            }
            Change::Delete { number } => {
                pka_event::delete_for_episode(&mut *tx, number).await?;
                pka_youtube_details::delete_for_episode(&mut *tx, number).await?;
                pka_episode::delete(&mut *tx, number).await?;
            }
        }
    }

    tx.commit().await?;

    Ok(())
}

#[derive(Debug, Default)]
pub struct Summary {
    pub details: usize,
    pub moments: usize,
    pub missing: usize,
    pub found: usize,
    pub deleted: usize,
}

impl Summary {
    pub fn changed(&self) -> bool {
        self.details + self.moments + self.missing + self.found + self.deleted > 0
    }
}

fn summarise(changes: &[Change]) -> Summary {
    let mut summary = Summary::default();

    for change in changes {
        match change {
            Change::Details { .. } => summary.details += 1,
            Change::Moments { .. } => summary.moments += 1,
            Change::MissingSince { .. } => summary.missing += 1,
            Change::Found { .. } => summary.found += 1,
            Change::Delete { .. } => summary.deleted += 1,
        }
    }

    summary
}

/// Checks every stored episode against YouTube and applies what changed. If
/// YouTube can't be asked about every video, nothing is changed: a missing
/// answer must never read as a missing video.
pub async fn sync_with_youtube(repo: &Repo, yt_api: &YoutubeApi) -> anyhow::Result<Summary> {
    let episodes = pka_episode::sync_states(repo)
        .await
        .context("Failed to load episodes to sync")?;
    let mut moments: HashMap<i64, Vec<PkaEvent>> = HashMap::new();

    for moment in pka_event::all(repo)
        .await
        .context("Failed to load moments to sync")?
    {
        moments
            .entry(episode_key(moment.episode_number()))
            .or_default()
            .push(moment);
    }

    for episode_moments in moments.values_mut() {
        episode_moments.sort_unstable_by_key(PkaEvent::timestamp);
    }

    let ids = episodes
        .iter()
        .map(|episode| episode.video_id.as_str())
        .collect::<Vec<_>>();
    let videos = yt_api
        .get_videos(&ids)
        .await?
        .into_iter()
        .map(|item| {
            (
                item.id,
                Video {
                    title: item.snippet.title,
                    length_seconds: item.content_details.duration,
                    description: item.snippet.description,
                },
            )
        })
        .collect::<HashMap<_, _>>();
    let now = chrono::Utc::now().timestamp();
    let changes = plan(&episodes, &moments, &videos, now);
    let summary = summarise(&changes);

    if summary.changed() {
        apply(repo, changes)
            .await
            .context("Failed to save the sync")?;
    }

    info!(
        episodes = episodes.len(),
        details = summary.details,
        moments = summary.moments,
        missing = summary.missing,
        found = summary.found,
        deleted = summary.deleted,
        "Synced with YouTube"
    );

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: i64 = 24 * 60 * 60;
    const NOW: i64 = 1_800_000_000;

    fn episode(number: f32) -> SyncState {
        SyncState {
            number,
            upload_date: 0,
            video_id: format!("video{number}").into(),
            title: "Old title".into(),
            length_seconds: 3600,
            moments_locked: false,
            missing_since: None,
        }
    }

    fn video(description: &str) -> Video {
        Video {
            title: "Old title".into(),
            length_seconds: 3600,
            description: description.into(),
        }
    }

    fn moments(number: f32, description: &str) -> Vec<PkaEvent> {
        extract_pka_episode_events(number, description, &3600, &0).unwrap()
    }

    fn plan_one(
        episode: &SyncState,
        stored: Vec<PkaEvent>,
        video: Option<Video>,
        now: i64,
    ) -> Vec<Change> {
        let moments = HashMap::from([(episode_key(episode.number), stored)]);
        let videos = video
            .map(|video| HashMap::from([(episode.video_id.clone(), video)]))
            .unwrap_or_default();

        plan(std::slice::from_ref(episode), &moments, &videos, now)
    }

    #[test]
    fn test_nothing_changes_when_everything_matches() {
        let timeline = "0:00 Intro\n10:00 Kyle's bike";

        assert!(
            plan_one(
                &episode(500.0),
                moments(500.0, timeline),
                Some(video(timeline)),
                NOW
            )
            .is_empty()
        );
    }

    #[test]
    fn test_titles_lengths_and_moments_follow_youtube() {
        let changes = plan_one(
            &episode(540.0),
            moments(540.0, "0:10 PKA reacts to EDP445 being exposed"),
            Some(Video {
                title: "New title".into(),
                length_seconds: 3500,
                description: "0:10 PKA reacts to cupcake being exposed".into(),
            }),
            NOW,
        );

        assert_eq!(changes.len(), 2);
        assert!(
            matches!(&changes[0], Change::Details { title, length_seconds: 3500, .. } if title == "New title")
        );
        assert!(
            matches!(&changes[1], Change::Moments { moments, .. } if moments[0].description() == "PKA reacts to cupcake being exposed")
        );
    }

    #[test]
    fn test_moments_stay_when_the_description_has_no_timeline() {
        let changes = plan_one(
            &episode(200.0),
            moments(200.0, "0:00 Community-written moment"),
            Some(video("Episode 200 of the podcast, no timestamps")),
            NOW,
        );

        assert!(changes.is_empty());
    }

    #[test]
    fn test_locked_and_hand_added_episodes_keep_their_moments() {
        let mut locked = episode(134.0);

        locked.moments_locked = true;

        for episode in [locked, episode(524.2)] {
            let changes = plan_one(
                &episode,
                moments(episode.number, "0:00 Our own moment"),
                Some(video("0:00 YouTube's different moment")),
                NOW,
            );

            assert!(changes.is_empty(), "{}", episode.number);
        }
    }

    #[test]
    fn test_a_missing_video_is_hidden_then_deleted_after_a_week() {
        let mut missing = episode(0.0);

        assert_eq!(
            plan_one(&missing, Vec::new(), None, NOW),
            vec![Change::MissingSince {
                number: 0.0,
                since: NOW
            }]
        );

        missing.missing_since = Some(NOW - 6 * DAY);
        assert!(plan_one(&missing, Vec::new(), None, NOW).is_empty());

        missing.missing_since = Some(NOW - 7 * DAY);
        assert_eq!(
            plan_one(&missing, Vec::new(), None, NOW),
            vec![Change::Delete { number: 0.0 }]
        );
    }

    #[test]
    fn test_a_video_that_comes_back_is_no_longer_missing() {
        let mut returned = episode(700.0);
        let timeline = "0:00 Intro";

        returned.missing_since = Some(NOW - 3 * DAY);

        assert_eq!(
            plan_one(
                &returned,
                moments(700.0, timeline),
                Some(video(timeline)),
                NOW
            ),
            vec![Change::Found { number: 700.0 }]
        );
    }
}
