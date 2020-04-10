use regex::RegexSetBuilder;

use crate::conduit::pka_episode;
use crate::models::search::{PkaEpisodeSearchResult, PkaEventSearchResult};
use crate::ALL_PKA_EVENTS;
use crate::{Repo, Result};

pub trait Searchable {
    fn field_to_match(&self) -> &str;
}

pub async fn search_episode(state: &Repo, query: &str) -> Result<Vec<PkaEpisodeSearchResult>> {
    let query = query.trim();

    let all_episodes = pka_episode::all_with_yt_details(&state).await?;

    if query != "" {
        search(query, &all_episodes)
    } else {
        Ok(all_episodes)
    }
}

pub async fn search_events(query: &str) -> Result<Vec<PkaEventSearchResult>> {
    let query = query.trim();

    if query.len() > 2 {
        let all_events = ALL_PKA_EVENTS.read().await;

        search(query, &*all_events)
    } else {
        Ok(Vec::new())
    }
}

fn search<T, R>(query: &str, items: &[T]) -> Result<Vec<R>>
where
    T: Searchable + Clone,
    R: std::cmp::Ord + From<T>,
{
    let queries = query
        .split(' ')
        .map(|q| regex::escape(q))
        .collect::<Vec<String>>();

    let all_regex_new = RegexSetBuilder::new(&queries)
        .case_insensitive(true)
        .build()?;

    let mut results: Vec<R> = items
        .iter()
        .filter(|i| {
            all_regex_new
                .matches(i.field_to_match())
                .iter()
                .count()
                == queries.len()
        })
        .map(|res| R::from(res.to_owned()))
        .collect();

    results.sort();

    Ok(results)
}
