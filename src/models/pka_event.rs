use std::cmp::Ordering;

use compact_str::CompactString;
use float_ord::FloatOrd;
use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::search::Searchable;

#[derive(Clone, Debug, Serialize, FromRow, ToSchema)]
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

impl AsRef<PkaEvent> for &PkaEvent {
    fn as_ref(&self) -> &PkaEvent {
        self
    }
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

impl Searchable for PkaEvent {
    fn field_to_match(&self) -> &str {
        self.description()
    }
}

impl std::cmp::Ord for PkaEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        FloatOrd(self.episode_number()).cmp(&FloatOrd(other.episode_number()))
    }
}

impl std::cmp::PartialOrd for PkaEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::PartialEq for PkaEvent {
    fn eq(&self, other: &Self) -> bool {
        FloatOrd(self.episode_number()).eq(&FloatOrd(other.episode_number()))
    }
}

impl std::cmp::Eq for PkaEvent {}
