#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use std::env;
use std::sync::Arc;

use axum::http::{header, Method};
use axum::routing::get;
use axum::{Json, Router};
use dotenv::dotenv;
use mimalloc::MiMalloc;
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::app_state::AppState;
use crate::conduit::sqlite::pka_event;
use crate::models::errors::ApiError;
use crate::models::pka_event::PkaEvent;
use crate::redis_db::RedisDb;
use crate::routes::build_router;
use crate::workers::events::update_events;
use crate::workers::new_episode::latest_episode;

mod app_state;
mod conduit;
mod db;
mod docs;
mod extractors;
mod handlers;
mod models;
mod redis_db;
mod routes;
mod search;
mod updater;
mod workers;

type Result<T> = std::result::Result<T, ApiError>;
type Repo = SqlitePool;
type EventIndexType = Arc<RwLock<Box<[PkaEvent]>>>;

lazy_static! {
    static ref YT_API_KEY: Arc<RwLock<String>> = Arc::new(RwLock::new(String::new()));
    static ref PKA_EVENTS_INDEX: EventIndexType = Arc::new(RwLock::new(Box::default()));
}

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() {
    dotenv().ok();

    env::set_var("RUST_LOG", "INFO");

    pretty_env_logger::init_timed();

    let redis_client: Arc<RedisDb> = Arc::new(
        RedisDb::new("redis://redis:6379")
            .await
            .expect("Failed to connect to redis."),
    );

    let db_pool: Arc<Repo> = Arc::new(
        db::create_pool(&env::var("DATABASE_URL").expect("'DATABASE_URL' is not set"))
            .await
            .expect("Failed to create SQLite pool"),
    );

    {
        *YT_API_KEY.write().await = env::var("YT_API_KEY").expect("'YT_API_KEY' is not set.");

        let all_events = pka_event::all(db_pool.as_ref())
            .await
            .expect("Failed to add all PKA events");

        *PKA_EVENTS_INDEX.write().await = all_events.into_boxed_slice();
    }

    let worker_state = || db_pool.clone();

    tokio::task::spawn(latest_episode(worker_state()));
    tokio::task::spawn(update_events(worker_state()));

    let app_state = AppState::new(db_pool.clone(), redis_client.clone());
    let openapi = Arc::new(docs::openapi());
    let docs_router = Router::new().route("/openapi.json", {
        let openapi = openapi.clone();

        get(move || async { Json(openapi) })
    });

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_origin(AllowOrigin::predicate(|_, _| true))
        .allow_credentials(true);

    let app = build_router()
        .merge(docs_router)
        .with_state(app_state)
        .layer(cors);

    let listener = TcpListener::bind("0.0.0.0:1234")
        .await
        .expect("Failed to bind listener");

    axum::serve(listener, app).await.expect("server error");
}
