use anyhow::Context;
use dotenv::dotenv;
use mimalloc::MiMalloc;
use sqlx::SqlitePool;
use std::sync::LazyLock;
use tokio::net::TcpListener;
use tower_http::compression::CompressionLayer;
use tracing::info;
use tracing_subscriber::prelude::*;

use crate::config::Config;
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
mod routes;
mod search;
mod startup;
mod updater;
mod workers;
mod yt_api_key;

type Repo = SqlitePool;

static YT_API_KEY: LazyLock<YtApiKey> = LazyLock::new(YtApiKey::default);

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

    // The episode list and broad searches are tens of kilobytes of JSON.
    let app = app
        .with_state(app_state)
        .layer(cors)
        .layer(CompressionLayer::new());

    let listener = TcpListener::bind(&config.bind_address)
        .await
        .context("Failed to bind listener")?;

    info!(address = %config.bind_address, "HTTP server listening");

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}

fn init_tracing() {
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| tracing_subscriber::EnvFilter::try_new("info"))
        .unwrap();

    let registry = tracing_subscriber::registry().with(filter_layer);

    if std::env::var("RUST_LOG_FORMAT").unwrap_or_default() == "json" {
        registry
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        registry.with(tracing_subscriber::fmt::layer()).init();
    }
}
