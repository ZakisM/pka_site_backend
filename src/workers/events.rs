use std::sync::Arc;

use tokio::time::{self, Duration};
use tracing::{error, info};

use crate::search::refresh_index;
use crate::Repo;

pub async fn update_search_index(state: Arc<Repo>) {
    let mut ticker = time::interval(Duration::from_secs(60));

    loop {
        ticker.tick().await;

        info!("Refreshing the search index...");

        if let Err(e) = refresh_index(&state).await {
            error!("update_search_index error: {:#}", e);
        }
    }
}
