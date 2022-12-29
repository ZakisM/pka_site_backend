use std::collections::HashSet;

use rayon::prelude::*;
use regex::{Regex, RegexSetBuilder};

use crate::conduit::redis::event_cache;
use crate::conduit::sqlite::pka_episode;
use crate::flatbuffers::pka_event::flatbuff_from_pka_events;
use crate::models::pka_event::PkaEvent;
use crate::models::search::PkaEpisodeSearchResult;
use crate::redis_db::RedisDb;
use crate::PKA_EVENTS_INDEX;
use crate::{Repo, Result};

pub trait Searchable {
    fn field_to_match(&self) -> &str;
}

pub async fn search_episode(state: &Repo, query: &str) -> Result<Vec<PkaEpisodeSearchResult>> {
    let query = query.trim();

    let all_episodes = pka_episode::all_with_yt_details(state).await?;

    if !query.is_empty() {
        search(query, &all_episodes)
    } else {
        Ok(all_episodes)
    }
}

pub async fn search_events(redis: &RedisDb, query: &str) -> Result<Vec<u8>> {
    let query = query.trim();

    let redis_tag = "EVENTS";

    if query.len() > 1 {
        match event_cache::get(redis, redis_tag, query.to_owned()).await {
            Ok(results) => Ok(results),
            Err(_) => {
                let all_events = PKA_EVENTS_INDEX.read().await;

                let events: Vec<&PkaEvent> = search_index(query, &all_events);
                let results = flatbuff_from_pka_events(events);

                event_cache::set(redis, redis_tag, query.to_owned(), results.as_slice()).await?;

                Ok(results)
            }
        }
    } else {
        Ok(Vec::new())
    }
}

pub fn create_index<T>(items: Vec<T>) -> Vec<(Vec<String>, T)>
where
    T: Searchable,
{
    lazy_static! {
        static ref WORD_REGEX: Regex =
            Regex::new(r"([^\s]+)").expect("Failed to create WORD_REGEX.");
    }

    items
        .into_iter()
        .map(|evt| {
            let mut searchable_terms_set: HashSet<String> = HashSet::new();

            for c in WORD_REGEX.find_iter(evt.field_to_match()) {
                searchable_terms_set.insert(c.as_str().to_lowercase());
            }

            let searchable_terms = searchable_terms_set.into_iter().collect();

            (searchable_terms, evt)
        })
        .collect()
}

fn search_index<'a, T>(query: &str, index: &'a [(Vec<String>, T)]) -> Vec<&'a T>
where
    T: Searchable + Sync + Send + Ord,
{
    let query = query.to_lowercase();

    let queries = query.split(' ').collect::<Vec<&str>>();

    let mut results: Vec<&T> = index
        .par_iter()
        .filter(|(ids, _)| queries.iter().all(|q| ids.iter().any(|i| i.contains(q))))
        .map(|(_, evt)| evt)
        .collect();

    results.sort();

    results
}

fn search<T, R>(query: &str, items: &[T]) -> Result<Vec<R>>
where
    T: Searchable + Clone,
    R: std::cmp::Ord + From<T>,
{
    let queries = query.split(' ').map(regex::escape).collect::<Vec<String>>();

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
