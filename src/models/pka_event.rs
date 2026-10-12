use compact_str::CompactString;
use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, Serialize, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PkaEvent {
    #[serde(skip_serializing)]
    #[schema(value_type = String)]
    pub event_id: CompactString,
    #[serde(skip_serializing)]
    pub episode_number: f32,
    pub timestamp: i32,
    #[schema(value_type = String)]
    pub description: CompactString,
    pub length_seconds: i32,
    pub upload_date: i64,
}

impl PkaEvent {
    pub fn new(
        event_id: CompactString,
        episode_number: f32,
        timestamp: i32,
        description: CompactString,
        length_seconds: i32,
        upload_date: i64,
    ) -> Self {
        PkaEvent {
            event_id,
            episode_number,
            timestamp,
            description,
            length_seconds,
            upload_date,
        }
    }

    pub fn episode_number(&self) -> f32 {
        self.episode_number
    }

    pub fn event_id(&self) -> CompactString {
        self.event_id.to_owned()
    }

    pub fn timestamp(&self) -> i32 {
        self.timestamp
    }

    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    pub fn length_seconds(&self) -> i32 {
        self.length_seconds
    }

    pub fn upload_date(&self) -> i64 {
        self.upload_date
    }
}

/// A moment with the episode it belongs to, for the random-moment link.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RandomPkaEvent {
    pub episode_number: f32,
    pub timestamp: i32,
    #[schema(value_type = String)]
    pub description: CompactString,
    pub length_seconds: i32,
    pub upload_date: i64,
}

impl From<&PkaEvent> for RandomPkaEvent {
    fn from(event: &PkaEvent) -> Self {
        Self {
            episode_number: event.episode_number(),
            timestamp: event.timestamp(),
            description: event.description.clone(),
            length_seconds: event.length_seconds(),
            upload_date: event.upload_date(),
        }
    }
}
