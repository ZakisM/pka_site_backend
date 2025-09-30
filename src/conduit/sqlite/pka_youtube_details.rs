use crate::models::pka_youtube_details::PkaYoutubeDetails;
use crate::Repo;

pub async fn insert(repo: &Repo, details: PkaYoutubeDetails) -> Result<(), sqlx::Error> {
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
