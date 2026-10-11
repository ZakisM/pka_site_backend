use std::str::FromStr;
use std::time::Duration;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

pub async fn create_pool(database_url: &str) -> sqlx::Result<SqlitePool> {
    let connect_options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(30));

    // Requests are answered from the in-memory catalog, so only the updater
    // And the catalog rebuild ever use the pool.
    SqlitePoolOptions::new()
        .max_connections(2)
        .min_connections(1)
        .idle_timeout(Duration::from_secs(60))
        .connect_with(connect_options)
        .await
}
