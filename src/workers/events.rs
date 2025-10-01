use std::sync::Arc;

use tokio::time::{self, Duration};
use tracing::{error, info};

use crate::conduit::sqlite::pka_event;
use crate::Repo;
use crate::PKA_EVENTS_INDEX;

pub async fn update_events(state: Arc<Repo>) {
    let mut ticker = time::interval(Duration::from_secs(60));

    loop {
        ticker.tick().await;

        info!("Updating all events...");

        match pka_event::all(&state).await {
            Ok(events) => {
                *PKA_EVENTS_INDEX.write().await = events.into_boxed_slice();
            }
            Err(e) => error!("get_latest_worker error: {}", e),
        }
    }
}
