//! The one background task: keeping the archive and the catalog current.

use std::time::Duration;

use tokio::time::{self, Instant, MissedTickBehavior};
use tracing::{error, info};

use crate::Repo;
use crate::catalog::{self, SharedCatalog};
use crate::updater::pka::load_new_episodes;
use crate::updater::youtube_api::YoutubeApi;

const CHECK_EVERY: Duration = Duration::from_secs(5 * 60);
// New episodes are picked up as they're saved; this catches edits made to the
// Database by hand.
const REFRESH_AT_LEAST_EVERY: Duration = Duration::from_secs(60 * 60);

/// Checks YouTube for new episodes every few minutes, and rebuilds the catalog
/// When one has been added, or when it's an hour old regardless.
pub async fn keep_current(repo: Repo, yt_api: YoutubeApi, catalog: SharedCatalog) {
    let mut ticker = time::interval(CHECK_EVERY);
    // Startup has just loaded the catalog.
    let mut refreshed_at = Instant::now();

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

        if added == 0 && refreshed_at.elapsed() < REFRESH_AT_LEAST_EVERY {
            continue;
        }

        info!(added, "Refreshing the catalog");

        match catalog::refresh(&repo, &catalog).await {
            Ok(()) => refreshed_at = Instant::now(),
            Err(e) => error!("Refreshing the catalog failed: {e:#}"),
        }
    }
}
