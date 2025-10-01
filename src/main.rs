use std::sync::Arc;

use anyhow::Context;
use dotenv::dotenv;
use mimalloc::MiMalloc;
use sqlx::SqlitePool;
use std::sync::LazyLock;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::info;
use tracing_subscriber::prelude::*;

use crate::config::Config;
use crate::models::pka_event::PkaEvent;
use crate::routes::build_router;
use crate::yt_api_key::YtApiKey;

mod app_state;
mod conduit;
mod config;
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
mod yt_api_key;

type Repo = SqlitePool;
type EventIndexType = Arc<RwLock<Box<[PkaEvent]>>>;

static YT_API_KEY: LazyLock<YtApiKey> = LazyLock::new(YtApiKey::default);
static PKA_EVENTS_INDEX: LazyLock<EventIndexType> =
    LazyLock::new(|| Arc::new(RwLock::new(Box::default())));

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{err:?}");

        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    dotenv().ok();

    init_tracing();

    let config = Config::from_env().context("Failed to load configuration")?;

    let startup::InitializedApp { app_state, cors } = startup::initialize(&config)
        .await
        .context("Failed to initialize application state")?;

    let mut app = build_router();
    if config.expose_openapi {
        app = app.merge(routes::docs::router());
    }

    let app = app.with_state(app_state).layer(cors);

    let listener = TcpListener::bind(&config.bind_address)
        .await
        .context("Failed to bind listener")?;

    info!(address = %config.bind_address, "HTTP server listening");

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
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
