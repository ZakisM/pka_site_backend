use anyhow::Context;
use dotenvy::dotenv;
use mimalloc::MiMalloc;
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::prelude::*;

use crate::config::Config;

mod app_state;
mod catalog;
mod conduit;
mod config;
mod db;
mod docs;
mod extractors;
mod handlers;
mod models;
mod routes;
mod startup;
mod updater;
mod workers;

#[cfg(test)]
mod http_tests;

type Repo = SqlitePool;

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

    let app_state = startup::initialize(&config)
        .await
        .context("Failed to initialize application state")?;
    let app = routes::app(app_state, config.expose_openapi);

    let listener = TcpListener::bind(&config.bind_address)
        .await
        .context("Failed to bind listener")?;

    info!(address = %config.bind_address, "HTTP server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Server error")?;

    info!("HTTP server stopped");

    Ok(())
}

/// Resolves on Ctrl+C or SIGTERM. In a container the app is PID 1, which the
/// Kernel gives no default SIGTERM handling: without this, `docker stop` (and
/// Every redeploy) waited out its 10 second grace period and then killed the
/// Process mid-request.
async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            tracing::error!("Failed to listen for Ctrl+C: {err}");
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(err) => {
                tracing::error!("Failed to listen for SIGTERM: {err}");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    info!("Shutting down: finishing in-flight requests");
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
