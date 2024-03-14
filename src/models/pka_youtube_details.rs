use compact_str::CompactString;
use serde::Serialize;

use crate::models::diesel_f32::DieselF32;
use crate::models::pka_episode::PkaEpisode;
use crate::schema::pka_youtube_details;

#[derive(Debug, Serialize, Insertable, Queryable, Identifiable, Associations)]
#[serde(rename_all = "camelCase")]
#[diesel(primary_key(video_id), belongs_to(PkaEpisode, foreign_key = episode_number), table_name = pka_youtube_details)]
pub struct PkaYoutubeDetails {
    video_id: CompactString,
    #[serde(skip_serializing)]
    episode_number: DieselF32,
    title: CompactString,
    length_seconds: i32,
}

impl PkaYoutubeDetails {
    pub fn new(
        video_id: CompactString,
        episode_number: f32,
        title: CompactString,
        length_seconds: i32,
    ) -> Self {
        PkaYoutubeDetails {
            video_id,
            episode_number: DieselF32(episode_number),
            title,
            length_seconds,
        }
    }

    pub fn length_seconds(&self) -> i32 {
        self.length_seconds
    }

    pub fn episode_number(&self) -> f32 {
        self.episode_number.0
    }
}
