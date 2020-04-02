use std::cmp::Ordering;

use float_ord::FloatOrd;
use serde::{Deserialize, Serialize};

use crate::models::pka_event::PkaEvent;

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
