use compact_str::CompactString;
use serde::Serialize;

use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PkaYoutubeDetails {
    pub video_id: CompactString,
    #[serde(skip_serializing)]
    pub episode_number: f32,
    pub title: CompactString,
    pub length_seconds: i32,
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
            episode_number,
            title,
            length_seconds,
        }
    }
}
