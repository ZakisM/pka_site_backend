use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};

use crate::models::pka_event::PkaEvent;

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SearchParams {
    /// Words to find; every one must appear in a moment or episode title.
    #[param(value_type = String)]
    pub q: CompactString,
    #[serde(default)]
    pub sort: SearchSort,
    /// Zero-based page of episode groups.
    #[serde(default)]
    pub page: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SearchSort {
    /// Newest episode first.
    #[default]
    Newest,
    /// Episodes with the most matching moments first, newest breaking ties.
    Matches,
}

/// An episode as lists and search results show it.
#[derive(Clone, Debug, Serialize, FromRow, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EpisodeSummary {
    pub episode_number: f32,
    pub upload_date: i64,
    pub title: String,
    pub length_seconds: i32,
    pub video_id: String,
    pub event_count: i32,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub timestamp: i32,
    #[schema(value_type = String)]
    pub description: CompactString,
    pub length_seconds: i32,
}

impl From<&PkaEvent> for SearchHit {
    fn from(event: &PkaEvent) -> Self {
        Self {
            timestamp: event.timestamp(),
            description: event.description.clone(),
            length_seconds: event.length_seconds(),
        }
    }
}

/// One episode in the results: its first few matching moments in order, and
/// How many matched in total.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchGroup {
    pub episode: EpisodeSummary,
    pub match_count: usize,
    pub hits: Vec<SearchHit>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(rename_all = "camelCase")]
pub struct RelatedParams {
    /// The moment's episode number.
    pub episode: f32,
    /// The moment's start, in seconds.
    pub timestamp: i32,
}

/// A moment in another episode about the same thing, with the words they share.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RelatedMoment {
    pub episode: EpisodeSummary,
    pub timestamp: i32,
    #[schema(value_type = String)]
    pub description: CompactString,
    pub length_seconds: i32,
    #[schema(value_type = Vec<String>)]
    pub shared_terms: Vec<CompactString>,
}

#[derive(Debug, Default, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchPage {
    pub total_moments: usize,
    pub total_episodes: usize,
    pub groups: Vec<SearchGroup>,
    pub next_page: Option<usize>,
}
