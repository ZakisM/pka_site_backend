#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_newtype;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use std::env;
use std::sync::Arc;

use diesel::SqliteConnection;
use dotenv::dotenv;
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
use crate::search::pka_search::create_index;
use crate::workers::events::update_events;
use crate::workers::new_episode::latest_episode;

mod conduit;
mod db;
mod flatbuffers;
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
type EventIndexType = Arc<RwLock<Vec<(Vec<String>, PkaEvent)>>>;

lazy_static! {
    static ref YT_API_KEY: Arc<RwLock<String>> = Arc::new(RwLock::new(String::new()));
    static ref PKA_EVENTS_INDEX: EventIndexType = Arc::new(RwLock::new(Vec::new()));
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    env::set_var("RUST_LOG", "INFO");

    pretty_env_logger::init_timed();

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
        *YT_API_KEY.write().await = env::var("YT_API_KEY").expect("'YT_API_KEY' is not set.");

        let all_events = pka_event::all(&state)
            .await
            .expect("Failed to add all PKA events");

        *PKA_EVENTS_INDEX.write().await = create_index(all_events);
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
