use compact_str::CompactString;
use sqlx::SqliteExecutor;

use crate::models::pka_event::PkaEvent;

pub async fn all<'e>(repo: impl SqliteExecutor<'e>) -> Result<Vec<PkaEvent>, sqlx::Error> {
    sqlx::query_as!(
        PkaEvent,
        r#"SELECT
            event_id       AS "event_id: CompactString",
            episode_number AS "episode_number: f32",
            timestamp      AS "timestamp: i32",
            description    AS "description: CompactString",
            length_seconds AS "length_seconds: i32",
            upload_date    AS "upload_date: i64"
          FROM pka_event"#
    )
    .fetch_all(repo)
    .await
}

pub async fn insert<'e>(repo: impl SqliteExecutor<'e>, event: PkaEvent) -> Result<(), sqlx::Error> {
    let PkaEvent {
        event_id,
        episode_number,
        timestamp,
        description,
        length_seconds,
        upload_date,
    } = event;

    sqlx::query!(
        r#"INSERT INTO pka_event (event_id, episode_number, timestamp, description, length_seconds, upload_date)
           VALUES (?, ?, ?, ?, ?, ?)"#,
        event_id,
        episode_number,
        timestamp,
        description,
        length_seconds,
        upload_date
    )
    .execute(repo)
    .await?;

    Ok(())
}
