use std::cmp::Ordering;

use float_ord::FloatOrd;
use serde::Serialize;

use crate::models::diesel_f32::DieselF32;
use crate::models::pka_episode::PkaEpisode;
use crate::schema::pka_event;
use crate::search::pka_search::Searchable;

#[derive(Clone, Debug, Serialize, Insertable, Queryable, Associations, Identifiable)]
#[serde(rename_all = "camelCase")]
#[primary_key(event_id)]
#[belongs_to(PkaEpisode, foreign_key = "episode_number")]
#[table_name = "pka_event"]
pub struct PkaEvent {
    #[serde(skip_serializing)]
    event_id: String,
    #[serde(skip_serializing)]
    episode_number: DieselF32,
    timestamp: i32,
    description: String,
    length_seconds: i32,
    upload_date: i64,
}

impl PkaEvent {
    pub fn new(
        event_id: String,
        episode_number: f32,
        timestamp: i32,
        description: String,
        length_seconds: i32,
        upload_date: i64,
    ) -> Self {
        PkaEvent {
            event_id,
            episode_number: DieselF32(episode_number),
            timestamp,
            description,
            length_seconds,
            upload_date,
        }
    }

    pub fn episode_number(&self) -> f32 {
        self.episode_number.0
    }

    pub fn event_id(&self) -> String {
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
        FloatOrd(self.episode_number()).partial_cmp(&FloatOrd(other.episode_number()))
    }
}

impl std::cmp::PartialEq for PkaEvent {
    fn eq(&self, other: &Self) -> bool {
        FloatOrd(self.episode_number()).eq(&FloatOrd(other.episode_number()))
    }
}

impl std::cmp::Eq for PkaEvent {}
