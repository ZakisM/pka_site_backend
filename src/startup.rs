use std::sync::Arc;

use anyhow::Context;
use axum::http::{header, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::app_state::AppState;
use crate::config::Config;
use crate::search::refresh_index;
use crate::workers::events::update_search_index;
use crate::workers::new_episode::latest_episode;
use crate::{db, Repo, YT_API_KEY};

pub struct InitializedApp {
    pub app_state: AppState,
    pub cors: CorsLayer,
}

pub async fn initialize(config: &Config) -> anyhow::Result<InitializedApp> {
    let db_pool: Arc<Repo> = Arc::new(db::create_pool(&config.database_url).await?);

    refresh_index(db_pool.as_ref())
        .await
        .context("Failed to prime the search index")?;
    YT_API_KEY.set(config.yt_api_key.clone()).await;

    let worker_state = || db_pool.clone();

    tokio::task::spawn(latest_episode(worker_state()));
    tokio::task::spawn(update_search_index(worker_state()));

    let app_state = AppState::new(db_pool.clone());
    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_origin(AllowOrigin::predicate(|_, _| true))
        .allow_credentials(true);

    Ok(InitializedApp { app_state, cors })
}
