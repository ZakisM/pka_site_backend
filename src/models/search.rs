use std::cmp::Ordering;

use bitcode::Encode;
use compact_str::CompactString;
use float_ord::FloatOrd;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::{models::pka_event::PkaEvent, search::Searchable};

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    #[schema(value_type = String)]
    pub query: CompactString,
}

#[derive(Clone, Encode, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PkaEventSearchResult {
    pub episode_number: f32,
    pub timestamp: i32,
    // TODO: Use Cow<'_, str> when bitcode supports
    #[schema(value_type = String)]
    pub description: String,
    pub length_seconds: i32,
    pub upload_date: i64,
}

impl<T: AsRef<PkaEvent>> From<T> for PkaEventSearchResult {
    fn from(evt: T) -> Self {
        let evt = evt.as_ref();

        Self {
            episode_number: evt.episode_number(),
            timestamp: evt.timestamp(),
            description: evt.description().to_owned(),
            length_seconds: evt.length_seconds(),
            upload_date: evt.upload_date(),
        }
    }
}

impl std::cmp::Ord for PkaEventSearchResult {
    fn cmp(&self, other: &Self) -> Ordering {
        FloatOrd(self.episode_number).cmp(&FloatOrd(other.episode_number))
    }
}

impl std::cmp::PartialOrd for PkaEventSearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::PartialEq for PkaEventSearchResult {
    fn eq(&self, other: &Self) -> bool {
        FloatOrd(self.episode_number).eq(&FloatOrd(other.episode_number))
    }
}

impl std::cmp::Eq for PkaEventSearchResult {}

#[derive(Clone, Encode, Debug, Serialize, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PkaEpisodeSearchResult {
    pub episode_number: f32,
    pub upload_date: i64,
    pub title: String,
    pub length_seconds: i32,
}

impl Searchable for PkaEpisodeSearchResult {
    fn field_to_match(&self) -> &str {
        self.title.as_str()
    }
}

impl std::cmp::Ord for PkaEpisodeSearchResult {
    fn cmp(&self, other: &Self) -> Ordering {
        FloatOrd(self.episode_number).cmp(&FloatOrd(other.episode_number))
    }
}

impl std::cmp::PartialOrd for PkaEpisodeSearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::PartialEq for PkaEpisodeSearchResult {
    fn eq(&self, other: &Self) -> bool {
        FloatOrd(self.episode_number).eq(&FloatOrd(other.episode_number))
    }
}

impl std::cmp::Eq for PkaEpisodeSearchResult {}
