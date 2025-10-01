use compact_str::CompactString;
use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Clone, Debug, Serialize, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PkaYoutubeDetails {
    #[schema(value_type = String)]
    pub video_id: CompactString,
    #[serde(skip_serializing)]
    pub episode_number: f32,
    #[schema(value_type = String)]
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
