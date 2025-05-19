use std::sync::Arc;

use tokio::time;
use tokio::time::Duration;

use crate::conduit::sqlite::pka_event;
use crate::Repo;
use crate::PKA_EVENTS_INDEX;

pub async fn update_events(state: Arc<Repo>) {
    loop {
        tracing::info!("Updating all events...");

        match pka_event::all(&state).await {
            Ok(events) => {
                *PKA_EVENTS_INDEX.get().unwrap().write().await = events.into_boxed_slice();
            }
            Err(e) => tracing::error!("get_latest_worker error: {}", e),
        }

        //Update every minute.
        time::sleep(Duration::from_secs(60)).await;
    }
}
