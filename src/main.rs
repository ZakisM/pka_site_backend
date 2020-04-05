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
use warp::filters::BoxedFilter;
use warp::Filter;

use crate::db::SqDatabase;
use crate::handlers::errors::handle_rejection;
use crate::models::errors::ApiError;
use crate::routes::episode::episode_routes;
use crate::routes::search::search_routes;
use crate::updater::pka::spawn_get_latest_worker;

mod conduit;
mod db;
mod handlers;
mod models;
mod routes;
mod schema;
mod search;
mod updater;

type Result<T> = std::result::Result<T, ApiError>;
type Repo = db::SqDatabase<SqliteConnection>;
type StateFilter = BoxedFilter<(Arc<Repo>,)>;

lazy_static! {
    static ref YT_API_KEY: String =
        env::var("YT_API_KEY").expect("Youtube API key required to start.");
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init_timed();

    let state: Arc<Repo> = Arc::new(SqDatabase::new("./pka_db.sqlite3"));

    let worker_state = state.clone();

    tokio::task::spawn(spawn_get_latest_worker(worker_state));

    let state_filter: StateFilter = warp::any().map(move || state.clone()).boxed();

    let state_c = || state_filter.clone();

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST"])
        .allow_headers(vec!["authorization", "content-type"])
        .allow_credentials(true);

    let api = search_routes(state_c())
        .or(episode_routes(state_c()))
        .with(cors)
        .recover(handle_rejection);

    warp::serve(api).run(([0, 0, 0, 0], 3030)).await;
}
