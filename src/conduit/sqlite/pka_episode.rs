use compact_str::CompactString;
use sqlx::SqliteExecutor;

use crate::models::pka_episode::PkaEpisode;

pub async fn all<'e>(repo: impl SqliteExecutor<'e>) -> Result<Vec<PkaEpisode>, sqlx::Error> {
    sqlx::query_as!(
        PkaEpisode,
        r#"SELECT
            number        AS "number: f32",
            name          AS "name: CompactString",
            youtube_link  AS "youtube_link: CompactString",
            upload_date   AS "upload_date: i64"
          FROM pka_episode"#
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
