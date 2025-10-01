use compact_str::CompactString;
use rand::prelude::IndexedRandom;
use rand::rng;

use crate::models::pka_event::PkaEvent;
use crate::models::search::PkaEventSearchResult;
use crate::Repo;

pub async fn all(repo: &Repo) -> Result<Vec<PkaEvent>, sqlx::Error> {
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

pub async fn insert(repo: &Repo, event: PkaEvent) -> Result<(), sqlx::Error> {
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

pub async fn random_amount(repo: &Repo) -> Result<Option<PkaEventSearchResult>, sqlx::Error> {
    let mut all_events = all(repo).await?;

    all_events.retain(|e| {
        let des = e.description().to_lowercase();
        !des.contains("outro") && !des.contains("intro") && !des.contains("ad read")
    });

    let mut rng = rng();

    let res = all_events.choose(&mut rng).map(PkaEventSearchResult::from);

    Ok(res)
}
