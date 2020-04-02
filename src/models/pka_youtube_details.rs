use serde::Serialize;

use crate::models::diesel_f32::DieselF32;
use crate::models::pka_episode::PkaEpisode;
use crate::schema::pka_youtube_details;

#[derive(Debug, Serialize, Insertable, Queryable, Identifiable, Associations)]
#[serde(rename_all = "camelCase")]
#[primary_key(video_id)]
#[belongs_to(PkaEpisode, foreign_key = "episode_number")]
#[table_name = "pka_youtube_details"]
pub struct PkaYoutubeDetails {
    video_id: String,
    #[serde(skip_serializing)]
    episode_number: DieselF32,
    title: String,
    length_seconds: i16,
    average_rating: DieselF32,
}

impl PkaYoutubeDetails {
    pub fn new(
        video_id: String,
        episode_number: f32,
        title: String,
        length_seconds: i16,
        average_rating: f32,
    ) -> Self {
        PkaYoutubeDetails {
            video_id,
            episode_number: DieselF32(episode_number),
            title,
            length_seconds,
            average_rating: DieselF32(average_rating),
        }
    }

    pub fn episode_number(&self) -> f32 {
        self.episode_number.0
    }
}
