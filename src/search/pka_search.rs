use std::collections::BTreeMap;

use float_ord::FloatOrd;
use regex::Regex;
use reqwest::StatusCode;
use strsim::normalized_damerau_levenshtein;

use crate::conduit::{pka_episode, pka_event};
use crate::models::errors::ApiError;
use crate::models::pka_episode::PkaEpisode;
use crate::models::search::PkaEventSearchResult;
use crate::{Repo, Result};

type SearchResults = BTreeMap<PkaEpisode, FloatOrd<f64>>;

pub async fn search_episode(state: &Repo, query: &str) -> Result<Vec<PkaEpisode>> {
    if query != "" {
        let all_episodes = pka_episode::all(state).await?;

        let mut results: SearchResults = BTreeMap::new();

        let mut max = 0.0;

        for ep in all_episodes.into_iter() {
            let res = normalized_damerau_levenshtein(query, ep.name()).abs();

            if res >= max {
                results.insert(ep, FloatOrd(res));
                max = res;
            }
        }

        let results = trim_results(results, max);

        Ok(results)
    } else {
        Ok(Vec::new())
    }
}

pub async fn search_events(state: &Repo, query: &str) -> Result<Vec<PkaEventSearchResult>> {
    let query = query.trim();

    if query.len() > 2 {
        // using a regex that ignores case is ~50% quicker than calling to_lowercase in a loop.
        let all_regex = query
            .split(' ')
            .map(|q| Regex::new(format!("(?i){}", regex::escape(q)).as_str()))
            .collect::<std::result::Result<Vec<Regex>, _>>()
            .map_err(|_| ApiError::new("Invalid search!", StatusCode::BAD_REQUEST))?;

        let all_events = pka_event::all(state).await?;

        let mut results = Vec::new();

        for event in all_events.into_iter() {
            let mut found = true;

            all_regex.iter().for_each(|r| {
                if !r.is_match(event.description()) {
                    found = false;
                    return;
                }
            });

            if found {
                results.push(PkaEventSearchResult::from(event));
            }
        }

        results.sort();

        Ok(results)
    } else {
        Ok(Vec::new())
    }
}

//remove any results that aren't within 10% of the closest match
fn trim_results(results: SearchResults, max: f64) -> Vec<PkaEpisode> {
    results
        .into_iter()
        .filter(|(_, v)| v.0 >= max - 0.1)
        .map(|(ep, _)| ep)
        .collect::<Vec<PkaEpisode>>()
}
