use compact_str::CompactString;

use crate::models::pka_episode::PkaEpisode;
use crate::models::pka_episode_with_all::PkaEpisodeWithAll;
use crate::models::pka_event::PkaEvent;
use crate::models::pka_youtube_details::PkaYoutubeDetails;
use crate::models::search::PkaEpisodeSearchResult;
use crate::Repo;

pub async fn all(repo: &Repo) -> Result<Vec<PkaEpisode>, sqlx::Error> {
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

pub async fn find_youtube_link(repo: &Repo, id: f32) -> Result<CompactString, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT youtube_link AS "youtube_link: CompactString" FROM pka_episode WHERE number = ?"#,
        id
    )
    .fetch_one(repo)
    .await
}

pub async fn latest(repo: &Repo) -> Result<f32, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT number AS "number: f32" FROM pka_episode ORDER BY number DESC LIMIT 1"#
    )
    .fetch_one(repo)
    .await
}

pub async fn random(repo: &Repo) -> Result<f32, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT number AS "number: f32" FROM pka_episode ORDER BY RANDOM() LIMIT 1"#
    )
    .fetch_one(repo)
    .await
}

pub async fn all_with_yt_details(repo: &Repo) -> Result<Vec<PkaEpisodeSearchResult>, sqlx::Error> {
    sqlx::query_as!(
        PkaEpisodeSearchResult,
        r#"SELECT
            e.number         AS "episode_number: f32",
            e.upload_date    AS "upload_date: i64",
            y.title          AS "title: String",
            y.length_seconds AS "length_seconds: i32"
          FROM pka_episode e
          INNER JOIN pka_youtube_details y ON y.episode_number = e.number
          ORDER BY e.number DESC"#
    )
    .fetch_all(repo)
    .await
}

pub async fn find_with_all(repo: &Repo, id: f32) -> Result<PkaEpisodeWithAll, sqlx::Error> {
    let episode = sqlx::query_as!(
        PkaEpisode,
        r#"SELECT
            number       AS "number: f32",
            name         AS "name: CompactString",
            youtube_link AS "youtube_link: CompactString",
            upload_date  AS "upload_date: i64"
          FROM pka_episode
          WHERE number = ?"#,
        id
    )
    .fetch_one(repo)
    .await?;

    let events = sqlx::query_as!(
        PkaEvent,
        r#"SELECT
            event_id       AS "event_id: CompactString",
            episode_number AS "episode_number: f32",
            timestamp      AS "timestamp: i32",
            description    AS "description: CompactString",
            length_seconds AS "length_seconds: i32",
            upload_date    AS "upload_date: i64"
          FROM pka_event
          WHERE episode_number = ?
          ORDER BY timestamp ASC"#,
        id
    )
    .fetch_all(repo)
    .await?;

    let youtube_details = sqlx::query_as!(
        PkaYoutubeDetails,
        r#"SELECT
            video_id       AS "video_id: CompactString",
            episode_number AS "episode_number: f32",
            title          AS "title: CompactString",
            length_seconds AS "length_seconds: i32"
          FROM pka_youtube_details
          WHERE episode_number = ?
          LIMIT 1"#,
        id
    )
    .fetch_one(repo)
    .await?;

    Ok(PkaEpisodeWithAll::new(episode, youtube_details, events))
}

pub async fn insert(repo: &Repo, episode: PkaEpisode) -> Result<(), sqlx::Error> {
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
