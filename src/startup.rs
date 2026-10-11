use anyhow::Context;

use crate::app_state::AppState;
use crate::catalog::{self, SharedCatalog};
use crate::config::Config;
use crate::db;
use crate::updater::youtube_api::YoutubeApi;
use crate::workers::keep_current;

/// Opens the database, loads the catalog from it and starts keeping both
/// Current. Fails if the catalog can't be loaded: there'd be nothing to serve.
pub async fn initialize(config: &Config) -> anyhow::Result<AppState> {
    let repo = db::create_pool(&config.database_url).await?;
    let catalog = SharedCatalog::new(
        catalog::load(&repo)
            .await
            .context("Failed to load the catalog")?,
    );
    let yt_api = YoutubeApi::new(config.yt_api_key.clone())?;

    tokio::task::spawn(keep_current(repo, yt_api, catalog.clone()));

    Ok(AppState::new(catalog))
}
