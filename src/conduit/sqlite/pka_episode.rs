use compact_str::CompactString;
use sqlx::SqliteExecutor;

use crate::models::pka_episode::{PkaEpisode, SyncState};

pub async fn all<'e>(repo: impl SqliteExecutor<'e>) -> Result<Vec<PkaEpisode>, sqlx::Error> {
    sqlx::query_as!(
        PkaEpisode,
        r#"SELECT
            number        AS "number: f32",
            name          AS "name: CompactString",
            youtube_link  AS "youtube_link: CompactString",
            upload_date   AS "upload_date: i64"
          FROM pka_episode
          WHERE missing_since IS NULL"#
    )
    .fetch_all(repo)
    .await
}

pub async fn latest<'e>(repo: impl SqliteExecutor<'e>) -> Result<f32, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT number AS "number: f32" FROM pka_episode ORDER BY number DESC LIMIT 1"#
    )
    .fetch_one(repo)
    .await
}

pub async fn insert<'e>(
    repo: impl SqliteExecutor<'e>,
    episode: PkaEpisode,
) -> Result<(), sqlx::Error> {
    let PkaEpisode {
        number,
        name,
        youtube_link,
        upload_date,
    } = episode;

    sqlx::query!(
        r#"INSERT INTO pka_episode (number, name, youtube_link, upload_date)
           VALUES (?, ?, ?, ?)"#,
        number,
        name,
        youtube_link,
        upload_date
    )
    .execute(repo)
    .await?;

    Ok(())
}

// Every column is NOT NULL but missing_since, which SQLite can't infer across
// The join, hence the overrides.
pub async fn sync_states<'e>(repo: impl SqliteExecutor<'e>) -> Result<Vec<SyncState>, sqlx::Error> {
    sqlx::query_as!(
        SyncState,
        r#"SELECT
            e.number         AS "number!: f32",
            e.upload_date    AS "upload_date!: i64",
            y.video_id       AS "video_id!: CompactString",
            y.title          AS "title!: CompactString",
            y.length_seconds AS "length_seconds!: i32",
            e.moments_locked AS "moments_locked!: bool",
            e.missing_since  AS "missing_since: i64"
          FROM pka_episode e
          INNER JOIN pka_youtube_details y ON y.episode_number = e.number"#
    )
    .fetch_all(repo)
    .await
}

pub async fn set_missing_since<'e>(
    repo: impl SqliteExecutor<'e>,
    number: f32,
    since: Option<i64>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE pka_episode SET missing_since = ? WHERE number = ?",
        since,
        number
    )
    .execute(repo)
    .await?;

    Ok(())
}

/// The episode row only: its moments and video details go first.
pub async fn delete<'e>(repo: impl SqliteExecutor<'e>, number: f32) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM pka_episode WHERE number = ?", number)
        .execute(repo)
        .await?;

    Ok(())
}
