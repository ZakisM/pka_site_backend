//! The one background task: keeping the archive and the catalog current.

mod schedule;

use std::time::Duration;

use tokio::time::{self, Instant, MissedTickBehavior};
use tracing::{error, info};

use self::schedule::{Job, Now};
use crate::Repo;
use crate::catalog::{self, SharedCatalog};
use crate::updater::pka::load_new_episodes;
use crate::updater::sync::sync_with_youtube;
use crate::updater::youtube_api::YoutubeApi;

// How often the schedule is looked at; it costs nothing until a job is due.
const TICK: Duration = Duration::from_secs(60);
const CHECK_EVERY: Duration = Duration::from_secs(5 * 60);
// PKA adds an episode's timeline some hours after uploading it, and every
// Later episode waits behind it: checking every few minutes until then would
// Spend quota for nothing.
const AWAIT_TIMELINE_FOR: Duration = Duration::from_secs(3 * 60 * 60);
// New episodes are picked up as they're saved; this catches edits made to the
// Database by hand.
const REFRESH_AT_LEAST_EVERY: Duration = Duration::from_secs(60 * 60);
// YouTube's terms allow 30 days between checks of what's stored; daily keeps
// PKA's edits prompt and leaves room for a failed day or two.
const SYNC_EVERY: Duration = Duration::from_secs(24 * 60 * 60);
const SYNC_RETRY_AFTER: Duration = Duration::from_secs(60 * 60);

/// Checks YouTube for new episodes every few minutes and syncs every stored
/// Episode with it daily, rebuilding the catalog when either changed anything,
/// Or when it's an hour old regardless. Both keep to a schedule saved in the
/// Database (see `schedule`), so restarting never asks YouTube early.
pub async fn keep_current(repo: Repo, yt_api: YoutubeApi, catalog: SharedCatalog) {
    let mut ticker = time::interval(TICK);
    // Startup has just loaded the catalog.
    let mut refreshed_at = Instant::now();
    let now = Now::read();
    let mut check = Job::load(&repo, "new_episodes", CHECK_EVERY, AWAIT_TIMELINE_FOR, now).await;
    let mut sync = Job::load(&repo, "sync", SYNC_RETRY_AFTER, SYNC_EVERY, now).await;

    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        ticker.tick().await;

        let now = Now::read();
        let mut added = 0;
        let mut synced = false;

        if check.is_due(now) && check.claim(&repo, now).await {
            match load_new_episodes(&repo, &yt_api).await {
                Ok(found) => {
                    added = found.added;

                    if found.awaiting_timeline {
                        info!("Checking for new episodes again in three hours");
                        check
                            .reschedule(&repo, Now::read(), AWAIT_TIMELINE_FOR)
                            .await;
                    }
                }
                Err(e) => error!("Checking for new episodes failed: {e:#}"),
            }
        }

        // Read again: the check may have taken a while.
        let now = Now::read();

        if sync.is_due(now) && sync.claim(&repo, now).await {
            match sync_with_youtube(&repo, &yt_api).await {
                Ok(summary) => {
                    synced = summary.changed();
                    sync.reschedule(&repo, Now::read(), SYNC_EVERY).await;
                }
                Err(e) => error!("Syncing with YouTube failed: {e:#}"),
            }
        }

        if added == 0 && !synced && refreshed_at.elapsed() < REFRESH_AT_LEAST_EVERY {
            continue;
        }

        info!(added, synced, "Refreshing the catalog");

        match catalog::refresh(&repo, &catalog).await {
            Ok(()) => refreshed_at = Instant::now(),
            Err(e) => error!("Refreshing the catalog failed: {e:#}"),
        }
    }
}
