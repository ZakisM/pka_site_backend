use std::env;
use std::sync::Arc;

use axum::http::{header, Method, StatusCode};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::app_state::AppState;
use crate::conduit::sqlite::pka_event;
use crate::models::errors::ApiError;
use crate::redis_db::RedisDb;
use crate::workers::events::update_events;
use crate::workers::new_episode::latest_episode;
use crate::{db, Result, Repo, YT_API_KEY, PKA_EVENTS_INDEX};

pub struct InitializedApp {
    pub app_state: AppState,
    pub cors: CorsLayer,
}

pub async fn initialize() -> Result<InitializedApp> {
    let redis_client: Arc<RedisDb> = Arc::new(RedisDb::new("redis://redis:6379").await?);

    let database_url = env::var("DATABASE_URL")
        .map_err(|_| ApiError::new("'DATABASE_URL' is not set", StatusCode::INTERNAL_SERVER_ERROR))?;

    let db_pool: Arc<Repo> = Arc::new(db::create_pool(&database_url).await?);

    {
        let yt_key = env::var("YT_API_KEY")
            .map_err(|_| ApiError::new("'YT_API_KEY' is not set.", StatusCode::INTERNAL_SERVER_ERROR))?;

        *YT_API_KEY.write().await = yt_key;

        let all_events = pka_event::all(db_pool.as_ref())
            .await?;

        *PKA_EVENTS_INDEX.write().await = all_events.into_boxed_slice();
    }

    let worker_state = || db_pool.clone();

    tokio::task::spawn(latest_episode(worker_state()));
    tokio::task::spawn(update_events(worker_state()));

    let app_state = AppState::new(db_pool.clone(), redis_client.clone());
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_origin(AllowOrigin::predicate(|_, _| true))
        .allow_credentials(true);

    Ok(InitializedApp {
        app_state,
        cors,
    })
}
