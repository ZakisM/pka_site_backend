use std::sync::Arc;

use tokio::time::{self, Duration};
use tracing::{error, info};

use crate::updater::pka::load_new_episodes;
use crate::Repo;

pub async fn latest_episode(state: Arc<Repo>) {
    let mut ticker = time::interval(Duration::from_secs(300));

    loop {
        ticker.tick().await;

        if let Err(e) = load_new_episodes(&state).await {
            error!("get_latest_worker error: {}", e);
        } else {
            info!("Successfully finished looking for latest episodes.");
        }
    }
}
