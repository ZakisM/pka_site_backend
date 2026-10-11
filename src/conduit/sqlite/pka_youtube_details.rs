use compact_str::CompactString;
use sqlx::SqliteExecutor;

use crate::models::pka_youtube_details::PkaYoutubeDetails;

pub async fn all<'e>(repo: impl SqliteExecutor<'e>) -> Result<Vec<PkaYoutubeDetails>, sqlx::Error> {
    sqlx::query_as!(
        PkaYoutubeDetails,
        r#"SELECT
            video_id       AS "video_id: CompactString",
            episode_number AS "episode_number: f32",
            title          AS "title: CompactString",
            length_seconds AS "length_seconds: i32"
          FROM pka_youtube_details"#
    )
    .fetch_all(repo)
    .await
}

pub async fn insert<'e>(
    repo: impl SqliteExecutor<'e>,
    details: PkaYoutubeDetails,
) -> Result<(), sqlx::Error> {
    let PkaYoutubeDetails {
        video_id,
        episode_number,
        title,
        length_seconds,
    } = details;

    sqlx::query!(
        r#"INSERT INTO pka_youtube_details (video_id, episode_number, title, length_seconds)
           VALUES (?, ?, ?, ?)"#,
        video_id,
        episode_number,
        title,
        length_seconds
    )
    .execute(repo)
    .await?;

    Ok(())
}
