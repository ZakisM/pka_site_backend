use std::collections::HashSet;

use rayon::prelude::*;
use regex::{Regex, RegexSetBuilder};

use crate::conduit::redis::event_cache;
use crate::conduit::sqlite::pka_episode;
use crate::models::search::{PkaEpisodeSearchResult, PkaEventSearchResult};
use crate::redis_db::RedisDb;
use crate::PKA_EVENTS_INDEX;
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

pub async fn search_events(redis: &RedisDb, query: &str) -> Result<Vec<PkaEventSearchResult>> {
    let query = query.trim();

    let redis_tag = "EVENTS";

    if query.len() > 2 {
        match event_cache::get(&redis, redis_tag, query.to_owned()).await {
            Ok(results) => Ok(results),
            Err(_) => {
                let all_events = PKA_EVENTS_INDEX.read().await;

                let results = search_index(query, &*all_events)?;

                event_cache::set(&redis, redis_tag, query.to_owned(), results.to_vec()).await?;

                Ok(results)
            }
        }
    } else {
        Ok(Vec::new())
    }
}

pub fn create_index<T>(items: Vec<T>) -> Vec<(HashSet<String>, T)>
where
    T: Searchable,
{
    lazy_static! {
        static ref WORD_REGEX: Regex = Regex::new(r"(\w+)").expect("Failed to create WORD_REGEX.");
    }

    items
        .into_iter()
        .map(|evt| {
            let mut searchable_terms: HashSet<String> = HashSet::new();

            for cap in WORD_REGEX.captures_iter(evt.field_to_match()) {
                for c in cap.iter() {
                    searchable_terms.insert(
                        c.expect("Failed to extract word to create index")
                            .as_str()
                            .to_lowercase(),
                    );
                }
            }

            (searchable_terms, evt)
        })
        .collect()
}

fn search_index<T, R>(query: &str, index: &[(HashSet<String>, T)]) -> Result<Vec<R>>
where
    T: Searchable + Clone + Sync + Send,
    R: std::cmp::Ord + From<T> + Send,
{
    let queries = query
        .split(' ')
        .map(|q| q.to_lowercase())
        .collect::<Vec<String>>();

    let mut results: Vec<R> = index
        .par_iter()
        .filter(|(ids, _)| queries.iter().all(|q| ids.iter().any(|i| i.contains(q))))
        .map(|(_, evt)| R::from(evt.to_owned()))
        .collect();

    results.sort();

    Ok(results)
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
        .filter(|i| all_regex_new.matches(i.field_to_match()).iter().count() == queries.len())
        .map(|res| R::from(res.to_owned()))
        .collect();

    results.sort();

    Ok(results)
}
