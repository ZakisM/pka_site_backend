use aho_corasick::AhoCorasickBuilder;
use anyhow::Context;
use rayon::prelude::*;

use crate::conduit::redis::event_cache;
use crate::conduit::sqlite::pka_episode;
use crate::models::search::PkaEventSearchResult;
use crate::redis_db::RedisDb;
use crate::search::{Encodeable, Searchable};
use crate::PKA_EVENTS_INDEX;
use crate::{Repo, Result};

pub async fn search_episode(state: &Repo, query: &str) -> Result<Vec<u8>> {
    let all_episodes = pka_episode::all_with_yt_details(state)
        .await
        .context("Failed to load episodes for search")?;

    let results = search(query, &all_episodes)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    let results = results
        .as_bitcode_compressed()
        .await
        .context("Failed to compress episode search results")?;

    Ok(results)
}

pub async fn search_events(redis: &RedisDb, query: &str) -> Result<Vec<u8>> {
    let redis_tag = "EVENTS";

    match event_cache::get(redis, redis_tag, query.to_owned()).await {
        Ok(results) => Ok(results),
        Err(_) => {
            let all_events = PKA_EVENTS_INDEX.read().await;

            let events = search(query, &all_events);
            let results = events
                .into_iter()
                .map(PkaEventSearchResult::from)
                .collect::<Vec<_>>();

            let results = results
                .as_bitcode_compressed()
                .await
                .context("Failed to compress event search results")?;

            event_cache::set(redis, redis_tag, query.to_owned(), results.as_slice()).await?;

            Ok(results)
        }
    }
}

fn search<'a, T>(query: &str, items: &'a [T]) -> Vec<&'a T>
where
    T: Searchable + Ord + Send + Sync,
{
    let query = query.trim();

    if query.is_empty() {
        let mut res = items.iter().collect::<Vec<_>>();

        res.sort();

        return res;
    }

    let patterns = query.split_ascii_whitespace();

    // Set all the bits to 1 for pattern_id,
    // i.e 3 patterns = (1 << 3) - 1
    // So 0b0000_1000 - 0b0000_0001:
    // 1000 -
    // 0001 =
    // 0111
    // Which is 0b0000_0111 <- The three bits for the pattern we've "seen"
    let seen = (1_u64 << patterns.clone().count()) - 1;

    let ac = AhoCorasickBuilder::new()
        .match_kind(aho_corasick::MatchKind::LeftmostFirst)
        .ascii_case_insensitive(true)
        .build(patterns)
        .expect("Failed to build aho_corasick");

    let mut results = items
        .par_iter()
        .filter(|item| {
            // Copy the seen
            let mut curr_seen = seen;

            for m in ac.find_iter(item.field_to_match()) {
                let pattern_id = m.pattern().as_u64();

                // Set seen bit to 0
                // i.e if curr is 0b0000_0111 and we've just seen pattern 1
                // 0b0000_0001 << 1 = 0b0000_0010
                // !0b0000_0010 = 0b1111_1101
                // curr_seen & 0b1111_1101 will always set that bit to 0
                // and leave rest of bits untouched
                curr_seen &= !(1 << pattern_id);

                if curr_seen == 0 {
                    return true;
                }
            }

            false
        })
        .collect::<Vec<_>>();

    results.sort();

    results
}
