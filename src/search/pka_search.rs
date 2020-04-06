use regex::Regex;
use reqwest::StatusCode;

use crate::conduit::{pka_episode, pka_event};
use crate::models::errors::ApiError;
use crate::models::search::{PkaEpisodeSearchResult, PkaEventSearchResult};
use crate::{Repo, Result};

pub trait Searchable {
    fn field_to_match(&self) -> &str;
}

pub async fn search_episode(state: &Repo, query: &str) -> Result<Vec<PkaEpisodeSearchResult>> {
    let query = query.trim();

    let all_episodes = pka_episode::all_with_yt_details(&state).await?;

    if query != "" {
        search(query, all_episodes)
    } else {
        Ok(all_episodes)
    }
}

pub async fn search_events(state: &Repo, query: &str) -> Result<Vec<PkaEventSearchResult>> {
    let query = query.trim();

    if query.len() > 2 {
        let all_events = pka_event::all(state).await?;

        search(query, all_events)
    } else {
        Ok(Vec::new())
    }
}

fn search<T, R>(query: &str, items: Vec<T>) -> Result<Vec<R>>
where
    T: Searchable,
    R: std::cmp::Ord + From<T>,
{
    let all_regex = regex_from_query(query)?;

    let mut results = Vec::new();

    for item in items.into_iter() {
        let mut found = true;

        all_regex.iter().for_each(|r| {
            if !r.is_match(item.field_to_match()) {
                found = false;
                return;
            }
        });

        if found {
            results.push(R::from(item));
        }
    }

    results.sort();

    Ok(results)
}

// using a regex that ignores case is ~50% quicker than calling to_lowercase in a loop.
fn regex_from_query(query: &str) -> Result<Vec<Regex>> {
    Ok(query
        .split(' ')
        .map(|q| Regex::new(format!("(?i){}", regex::escape(q)).as_str()))
        .collect::<std::result::Result<Vec<Regex>, _>>()
        .map_err(|_| ApiError::new("Invalid search!", StatusCode::BAD_REQUEST))?)
}
