use std::sync::Arc;

use tokio::time;
use tokio::time::Duration;

use crate::updater::pka::get_latest_pka_episode_data;
use crate::Repo;

pub async fn latest_episode(state: Arc<Repo>) {
    loop {
        tracing::info!("Checking for latest episode...");

        if let Err(e) = get_latest_pka_episode_data(&state).await {
            tracing::error!("get_latest_worker error: {}", e);
        } else {
            tracing::info!("Successfully finished looking for latest episodes.");
        }

        //Check once every five minutes.
        time::sleep(Duration::from_secs(300)).await;
    }
}
