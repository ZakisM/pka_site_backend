use std::cmp::Ordering;

use float_ord::FloatOrd;
use serde::{Deserialize, Serialize};

use crate::models::pka_event::PkaEvent;
use crate::search::pka_search::Searchable;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    pub query: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PkaEventSearchResult {
    episode_number: f32,
    timestamp: i64,
    description: String,
}

impl From<PkaEvent> for PkaEventSearchResult {
    fn from(e: PkaEvent) -> Self {
        Self {
            episode_number: e.episode_number(),
            timestamp: e.timestamp(),
            description: e.description().to_owned(),
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
        FloatOrd(self.episode_number).partial_cmp(&FloatOrd(other.episode_number))
    }
}

impl std::cmp::PartialEq for PkaEventSearchResult {
    fn eq(&self, other: &Self) -> bool {
        FloatOrd(self.episode_number).eq(&FloatOrd(other.episode_number))
    }
}

impl std::cmp::Eq for PkaEventSearchResult {}

#[derive(Debug, Serialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct PkaEpisodeSearchResult {
    episode_number: f32,
    upload_date: i64,
    title: String,
}

impl Searchable for PkaEpisodeSearchResult {
    fn field_to_match(&self) -> &str {
        self.title.as_ref()
    }
}

impl std::cmp::Ord for PkaEpisodeSearchResult {
    fn cmp(&self, other: &Self) -> Ordering {
        FloatOrd(self.episode_number).cmp(&FloatOrd(other.episode_number))
    }
}

impl std::cmp::PartialOrd for PkaEpisodeSearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        FloatOrd(self.episode_number).partial_cmp(&FloatOrd(other.episode_number))
    }
}

impl std::cmp::PartialEq for PkaEpisodeSearchResult {
    fn eq(&self, other: &Self) -> bool {
        FloatOrd(self.episode_number).eq(&FloatOrd(other.episode_number))
    }
}

impl std::cmp::Eq for PkaEpisodeSearchResult {}
