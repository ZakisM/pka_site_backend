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
