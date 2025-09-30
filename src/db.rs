use std::str::FromStr;
use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;

pub async fn create_pool(database_url: &str) -> sqlx::Result<SqlitePool> {
    let connect_options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(30));

    SqlitePoolOptions::new()
        .max_connections(15)
        .min_connections(1)
        .idle_timeout(Duration::from_secs(60))
        .connect_with(connect_options)
        .await
}
