use std::sync::Arc;

use anyhow::Context;
use axum::http::{header, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::app_state::AppState;
use crate::conduit::sqlite::pka_event;
use crate::config::Config;
use crate::redis_db::RedisDb;
use crate::workers::events::update_events;
use crate::workers::new_episode::latest_episode;
use crate::{db, Repo, Result, PKA_EVENTS_INDEX, YT_API_KEY};

pub struct InitializedApp {
    pub app_state: AppState,
    pub cors: CorsLayer,
}

pub async fn initialize(config: &Config) -> Result<InitializedApp> {
    let redis_client: Arc<RedisDb> = Arc::new(RedisDb::new(&config.redis_url).await?);

    let db_pool: Arc<Repo> = Arc::new(db::create_pool(&config.database_url).await?);

    let all_events = pka_event::all(db_pool.as_ref())
        .await
        .context("Failed to prime event index")?;

    *PKA_EVENTS_INDEX.write().await = all_events.into_boxed_slice();
    YT_API_KEY.set(config.yt_api_key.clone()).await;

    let worker_state = || db_pool.clone();

    tokio::task::spawn(latest_episode(worker_state()));
    tokio::task::spawn(update_events(worker_state()));

    let app_state = AppState::new(db_pool.clone(), redis_client.clone());
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_origin(AllowOrigin::predicate(|_, _| true))
        .allow_credentials(true);

    Ok(InitializedApp { app_state, cors })
}
