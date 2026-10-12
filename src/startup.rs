use anyhow::Context;

use crate::app_state::AppState;
use crate::catalog::{self, SharedCatalog};
use crate::config::Config;
use crate::db;
use crate::updater::youtube_api::YoutubeApi;
use crate::workers::keep_current;

/// Opens the database (bringing its schema up to date), loads the catalog
/// From it and starts keeping both current. Fails if either fails: there'd be
/// Nothing to serve.
pub async fn initialize(config: &Config) -> anyhow::Result<AppState> {
    let repo = db::create_pool(&config.database_url).await?;

    // The schema changes ship with the code. The first migrations only create
    // Tables if missing, so a database made before migrations ran here takes
    // Them as already applied.
    sqlx::migrate!()
        .run(&repo)
        .await
        .context("Failed to update the database schema")?;
    let catalog = SharedCatalog::new(
        catalog::load(&repo)
            .await
            .context("Failed to load the catalog")?,
    );
    let yt_api = YoutubeApi::new(config.yt_api_key.clone())?;

    tokio::task::spawn(keep_current(repo, yt_api, catalog.clone()));

    Ok(AppState::new(catalog))
}
