use std::sync::Arc;

use dotenv::dotenv;
use mimalloc::MiMalloc;
use sqlx::SqlitePool;
use std::sync::LazyLock;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing_subscriber::prelude::*;

use crate::models::errors::ApiError;
use crate::models::pka_event::PkaEvent;
use crate::routes::build_router;

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
mod startup;
mod updater;
mod workers;

type Result<T> = std::result::Result<T, ApiError>;
type Repo = SqlitePool;
type EventIndexType = Arc<RwLock<Box<[PkaEvent]>>>;

static YT_API_KEY: LazyLock<Arc<RwLock<String>>> =
    LazyLock::new(|| Arc::new(RwLock::new(String::new())));
static PKA_EVENTS_INDEX: LazyLock<EventIndexType> =
    LazyLock::new(|| Arc::new(RwLock::new(Box::default())));

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() {
    dotenv().ok();

    init_tracing();

    let startup::InitializedApp { app_state, cors } = startup::initialize()
        .await
        .expect("Failed to initialize application state");

    let app = build_router().with_state(app_state).layer(cors);

    let listener = TcpListener::bind("0.0.0.0:1234")
        .await
        .expect("Failed to bind listener");

    axum::serve(listener, app).await.expect("server error");
}

fn init_tracing() {
    let fmt_layer = tracing_subscriber::fmt::layer();

    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| tracing_subscriber::EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
}
