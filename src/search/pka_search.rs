use ahash::AHashSet;
use aho_corasick::AhoCorasickBuilder;
use rayon::prelude::*;

use crate::conduit::redis::event_cache;
use crate::conduit::sqlite::pka_episode;
use crate::flatbuffers::pka_event::flatbuff_from_pka_events;
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
        let results = search(query, &all_episodes);

        Ok(results.into_iter().cloned().collect())
    } else {
        Ok(all_episodes)
    }
}

pub async fn search_events(redis: &RedisDb, query: &str) -> Result<Vec<u8>> {
    let query = query.trim();

    if query.is_empty() {
        return Ok(Vec::new());
    }

    let redis_tag = "EVENTS";

    match event_cache::get(redis, redis_tag, query.to_owned()).await {
        Ok(results) => Ok(results),
        Err(_) => {
            let all_events = PKA_EVENTS_INDEX.read().await;

            let events = search(query, &all_events);
            let results = flatbuff_from_pka_events(events);

            event_cache::set(redis, redis_tag, query.to_owned(), results.as_slice()).await?;

            Ok(results)
        }
    }
}

fn search<'a, T>(query: &str, items: &'a [T]) -> Vec<&'a T>
where
    T: Searchable + Ord + Send + Sync,
{
    let patterns = query.split_ascii_whitespace();
    let patterns_len = patterns.clone().count();

    let ac = AhoCorasickBuilder::new()
        .ascii_case_insensitive(true)
        .build(patterns)
        .expect("Failed to build aho_corasick");

    let mut results = items
        .par_iter()
        .filter(|item| {
            ac.find_iter(item.field_to_match())
                .fold(AHashSet::default(), |mut curr, next| {
                    curr.insert(next.pattern());
                    curr
                })
                .len()
                == patterns_len
        })
        .collect::<Vec<_>>();

    results.sort();

    results
}
