#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_newtype;

use std::env;
use std::sync::{Arc, OnceCell};

use diesel::SqliteConnection;
use dotenv::dotenv;
use mimalloc::MiMalloc;
use tokio::sync::RwLock;
use warp::filters::BoxedFilter;
use warp::Filter;

use crate::conduit::sqlite::pka_event;
use crate::db::SqDatabase;
use crate::handlers::errors::handle_rejection;
use crate::models::errors::ApiError;
use crate::models::pka_event::PkaEvent;
use crate::redis_db::RedisDb;
use crate::routes::episode::episode_routes;
use crate::routes::events::event_routes;
use crate::routes::search::search_routes;
use crate::routes::static_files::static_files_routes;
use crate::workers::events::update_events;
use crate::workers::new_episode::latest_episode;

mod conduit;
mod db;
mod handlers;
mod models;
mod redis_db;
mod routes;
mod schema;
mod search;
mod updater;
mod workers;

type Result<T> = std::result::Result<T, ApiError>;
type Repo = db::SqDatabase<SqliteConnection>;
type StateFilter = BoxedFilter<(Arc<Repo>,)>;
type RedisFilter = BoxedFilter<(Arc<RedisDb>,)>;
type EventIndexType = Arc<RwLock<Box<[PkaEvent]>>>;

pub static YT_API_KEY: OnceCell<Arc<RwLock<String>>> = OnceCell::new();
pub static PKA_EVENTS_INDEX: OnceCell<EventIndexType> = OnceCell::new();

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() {
    dotenv().ok();

    env::set_var("RUST_LOG", "INFO");

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_timer(tracing_subscriber::fmt::time::ChronoUtc::rfc3339())
        .init();

    // DB and Redis
    let redis_client: Arc<RedisDb> = Arc::new(
        RedisDb::new("redis://redis:6379")
            .await
            .expect("Failed to connect to redis."),
    );

    let state: Arc<Repo> = Arc::new(SqDatabase::new(
        &env::var("DATABASE_URL").expect("'DATABASE_URL' is not set"),
    ));

    {
        let api_key_val = env::var("YT_API_KEY").expect("'YT_API_KEY' is not set.");
        YT_API_KEY.set(Arc::new(RwLock::new(api_key_val))).expect("Failed to set YT_API_KEY global");

        let all_events_val = pka_event::all(&state)
            .await
            .expect("Failed to add all PKA events");
        PKA_EVENTS_INDEX.set(Arc::new(RwLock::new(all_events_val.into_boxed_slice()))).expect("Failed to set PKA_EVENTS_INDEX global");
    }

    // workers
    let worker_state = || state.clone();

    tokio::task::spawn(latest_episode(worker_state()));
    tokio::task::spawn(update_events(worker_state()));

    let state_filter: StateFilter = warp::any().map(move || state.clone()).boxed();
    let redis_filter: RedisFilter = warp::any().map(move || redis_client.clone()).boxed();

    let state_c = || state_filter.clone();
    let redis_c = || redis_filter.clone();

    let cors = warp::cors()
        .allow_methods(vec!["GET", "POST"])
        .allow_headers(vec!["authorization", "content-type"])
        .allow_credentials(true);

    let api = search_routes(state_c(), redis_c())
        .or(episode_routes(state_c()))
        .or(event_routes(state_c()))
        .or(static_files_routes(state_c()))
        .with(cors)
        .recover(handle_rejection);

    warp::serve(api).run(([0, 0, 0, 0], 1234)).await;
}
