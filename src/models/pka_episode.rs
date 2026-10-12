use compact_str::CompactString;
use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Clone, Debug, Serialize, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PkaEpisode {
    pub number: f32,
    #[schema(value_type = String)]
    pub name: CompactString,
    #[serde(skip_serializing)]
    #[schema(value_type = String)]
    pub youtube_link: CompactString,
    pub upload_date: i64,
}

impl PkaEpisode {
    pub fn new(
        number: f32,
        name: CompactString,
        youtube_link: CompactString,
        upload_date: i64,
    ) -> Self {
        Self {
            number,
            name,
            youtube_link,
            upload_date,
        }
    }
}

/// What the YouTube sync needs to know about a stored episode.
#[derive(Clone, Debug)]
pub struct SyncState {
    pub number: f32,
    pub upload_date: i64,
    pub video_id: CompactString,
    pub title: CompactString,
    pub length_seconds: i32,
    // Its moments are the site's own: never replaced from the description.
    pub moments_locked: bool,
    // Unix seconds when YouTube was first found without its video.
    pub missing_since: Option<i64>,
}
