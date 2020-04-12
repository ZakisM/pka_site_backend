use std::sync::Arc;

use tokio::time;
use tokio::time::Duration;

use crate::conduit::sqlite::pka_event;
use crate::search::pka_search::create_index;
use crate::Repo;
use crate::PKA_EVENTS_INDEX;

pub async fn update_events(state: Arc<Repo>) {
    loop {
        info!("Updating all events...");

        match pka_event::all(&state).await {
            Ok(events) => {
                *PKA_EVENTS_INDEX.write().await = create_index(events);
            }
            Err(e) => error!("get_latest_worker error: {}", e),
        }

        //Update every minute.
        time::delay_for(Duration::from_secs(60)).await;
    }
}
