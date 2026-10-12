//! The one background task: keeping the archive and the catalog current.

use std::time::Duration;

use tokio::time::{self, Instant, MissedTickBehavior};
use tracing::{error, info};

use crate::Repo;
use crate::catalog::{self, SharedCatalog};
use crate::updater::pka::load_new_episodes;
use crate::updater::sync::sync_with_youtube;
use crate::updater::youtube_api::YoutubeApi;

const CHECK_EVERY: Duration = Duration::from_secs(5 * 60);
// New episodes are picked up as they're saved; this catches edits made to the
// Database by hand.
const REFRESH_AT_LEAST_EVERY: Duration = Duration::from_secs(60 * 60);
// YouTube's terms allow 30 days between checks of what's stored; daily keeps
// PKA's edits prompt and leaves room for a failed day or two.
const SYNC_EVERY: Duration = Duration::from_secs(24 * 60 * 60);
const SYNC_RETRY_AFTER: Duration = Duration::from_secs(60 * 60);

/// Checks YouTube for new episodes every few minutes and syncs every stored
/// Episode with it daily, rebuilding the catalog when either changed anything,
/// Or when it's an hour old regardless.
pub async fn keep_current(repo: Repo, yt_api: YoutubeApi, catalog: SharedCatalog) {
    let mut ticker = time::interval(CHECK_EVERY);
    // Startup has just loaded the catalog; the first sync runs straight away.
    let mut refreshed_at = Instant::now();
    let mut next_sync = Instant::now();

    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        ticker.tick().await;

        let added = match load_new_episodes(&repo, &yt_api).await {
            Ok(added) => added,
            Err(e) => {
                error!("Checking for new episodes failed: {e:#}");
                0
            }
        };
        let mut synced = false;

        if Instant::now() >= next_sync {
            match sync_with_youtube(&repo, &yt_api).await {
                Ok(summary) => {
                    synced = summary.changed();
                    next_sync = Instant::now() + SYNC_EVERY;
                }
                Err(e) => {
                    error!("Syncing with YouTube failed: {e:#}");
                    next_sync = Instant::now() + SYNC_RETRY_AFTER;
                }
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
