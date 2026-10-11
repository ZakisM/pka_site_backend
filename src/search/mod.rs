use std::cmp::Reverse;
use std::collections::HashMap;
use std::sync::LazyLock;

use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use anyhow::Context;
use rayon::prelude::*;
use tokio::sync::RwLock;

use crate::conduit::sqlite::{pka_episode, pka_event};
use crate::models::pka_event::PkaEvent;
use crate::models::search::{EpisodeSummary, SearchGroup, SearchHit, SearchPage, SearchSort};
use crate::Repo;

pub const GROUPS_PER_PAGE: usize = 12;
pub const HITS_PER_GROUP: usize = 3;
// Beyond this a query is a paste, not a search; it also bounds the bitmask.
const MAX_TERMS: usize = 16;

/// Everything search reads, held in memory and swapped wholesale on refresh so
/// A request never sees episodes and moments from different snapshots.
#[derive(Default)]
pub struct SearchIndex {
    /// Newest first, which is also the default result order.
    pub episodes: Box<[EpisodeSummary]>,
    pub events: Box<[PkaEvent]>,
}

pub static SEARCH_INDEX: LazyLock<RwLock<SearchIndex>> = LazyLock::new(Default::default);

pub async fn refresh_index(repo: &Repo) -> anyhow::Result<()> {
    let episodes = pka_episode::all_summaries(repo)
        .await
        .context("Failed to load episodes for the search index")?;
    let events = pka_event::all(repo)
        .await
        .context("Failed to load events for the search index")?;

    *SEARCH_INDEX.write().await = SearchIndex {
        episodes: episodes.into_boxed_slice(),
        events: events.into_boxed_slice(),
    };

    Ok(())
}

/// Matches text containing every word of the query, case-insensitively.
struct Matcher {
    automaton: AhoCorasick,
    all_seen: u32,
}

impl Matcher {
    fn new(query: &str) -> Option<Self> {
        let mut terms = query
            .split_whitespace()
            .map(str::to_lowercase)
            .collect::<Vec<_>>();

        // A repeated word would leave its second bit unset forever.
        terms.sort_unstable();
        terms.dedup();
        terms.truncate(MAX_TERMS);

        if terms.is_empty() {
            return None;
        }

        // Overlapping, so "tay taylor" sees both words in "Taylor".
        let automaton = AhoCorasickBuilder::new()
            .ascii_case_insensitive(true)
            .match_kind(MatchKind::Standard)
            .build(&terms)
            .ok()?;

        Some(Self {
            automaton,
            all_seen: (1 << terms.len()) - 1,
        })
    }

    fn matches(&self, text: &str) -> bool {
        let mut seen = 0_u32;

        for found in self.automaton.find_overlapping_iter(text) {
            seen |= 1 << found.pattern().as_u32();

            if seen == self.all_seen {
                return true;
            }
        }

        false
    }
}

// Episode numbers can be fractional (524.1); compare them as tenths.
fn episode_key(number: f32) -> i64 {
    (f64::from(number) * 10.0).round() as i64
}

/// One page of results, grouped by episode: every episode with a matching
/// Moment, plus any whose title matches.
pub fn search(index: &SearchIndex, query: &str, sort: SearchSort, page: usize) -> SearchPage {
    let Some(matcher) = Matcher::new(query) else {
        return SearchPage::default();
    };

    let mut hits_by_episode: HashMap<i64, Vec<&PkaEvent>> = HashMap::new();

    for event in index
        .events
        .par_iter()
        .filter(|event| matcher.matches(event.description()))
        .collect::<Vec<_>>()
    {
        hits_by_episode
            .entry(episode_key(event.episode_number()))
            .or_default()
            .push(event);
    }

    let mut groups = index
        .episodes
        .iter()
        .filter_map(|episode| {
            let hits = hits_by_episode
                .remove(&episode_key(episode.episode_number))
                .unwrap_or_default();

            (!hits.is_empty() || matcher.matches(&episode.title)).then_some((episode, hits))
        })
        .collect::<Vec<_>>();

    if sort == SearchSort::Matches {
        // Stable, so equal counts keep the index's newest-first order.
        groups.sort_by_key(|(_, hits)| Reverse(hits.len()));
    }

    let total_moments = groups.iter().map(|(_, hits)| hits.len()).sum();
    let total_episodes = groups.len();
    let start = page.saturating_mul(GROUPS_PER_PAGE);

    let groups = groups
        .into_iter()
        .skip(start)
        .take(GROUPS_PER_PAGE)
        .map(|(episode, mut hits)| {
            hits.sort_by_key(|event| event.timestamp());

            SearchGroup {
                episode: episode.clone(),
                match_count: hits.len(),
                hits: hits
                    .into_iter()
                    .take(HITS_PER_GROUP)
                    .map(SearchHit::from)
                    .collect(),
            }
        })
        .collect();

    SearchPage {
        total_moments,
        total_episodes,
        groups,
        next_page: (start + GROUPS_PER_PAGE < total_episodes).then_some(page + 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn episode(number: f32, title: &str) -> EpisodeSummary {
        EpisodeSummary {
            episode_number: number,
            upload_date: 0,
            title: title.to_owned(),
            length_seconds: 14_400,
            video_id: format!("video{number}"),
            event_count: 0,
        }
    }

    fn event(number: f32, timestamp: i32, description: &str) -> PkaEvent {
        PkaEvent::new(
            format!("{number}-{timestamp}").into(),
            number,
            timestamp,
            description.into(),
            60,
            0,
        )
    }

    fn index() -> SearchIndex {
        SearchIndex {
            episodes: vec![
                episode(825.0, "PKA 825 W/ Harley: Redemption Of Wings"),
                episode(524.1, "Part 1 Confederate Talk"),
                episode(400.0, "PKA 400 w/Tucker - Taylor's Bible Story"),
            ]
            .into_boxed_slice(),
            events: vec![
                event(825.0, 1547, "Why Pinhead from Hellraiser is terrifying"),
                event(825.0, 30, "Taylor reads the sponsors"),
                event(400.0, 900, "Taylor's bible story"),
                event(400.0, 100, "Taylor and Kyle argue"),
                event(400.0, 50, "Taylor intro"),
                event(400.0, 5000, "Taylor outro"),
                event(524.1, 10, "Confederate talk part one"),
            ]
            .into_boxed_slice(),
        }
    }

    #[test]
    fn test_every_word_must_match_in_any_case() {
        let matcher = Matcher::new("KYLE taylor").unwrap();

        assert!(matcher.matches("Taylor and Kyle argue"));
        assert!(!matcher.matches("Taylor reads the sponsors"));
    }

    #[test]
    fn test_repeated_and_overlapping_words_still_match() {
        assert!(Matcher::new("kyle kyle").unwrap().matches("Kyle stories"));
        assert!(Matcher::new("tay taylor")
            .unwrap()
            .matches("Taylor's bible story"));
    }

    #[test]
    fn test_blank_query_returns_nothing() {
        let page = search(&index(), "   ", SearchSort::Newest, 0);

        assert_eq!(page.total_episodes, 0);
        assert!(page.groups.is_empty());
    }

    #[test]
    fn test_groups_by_episode_newest_first_with_hits_in_order() {
        let page = search(&index(), "taylor", SearchSort::Newest, 0);

        assert_eq!(page.total_moments, 5);
        assert_eq!(page.total_episodes, 2);
        assert_eq!(page.groups[0].episode.episode_number, 825.0);

        let older = &page.groups[1];

        assert_eq!(older.match_count, 4);
        assert_eq!(older.hits.len(), HITS_PER_GROUP);
        assert_eq!(
            older
                .hits
                .iter()
                .map(|hit| hit.timestamp)
                .collect::<Vec<_>>(),
            vec![50, 100, 900]
        );
    }

    #[test]
    fn test_most_matches_sort_puts_busiest_episode_first() {
        let page = search(&index(), "taylor", SearchSort::Matches, 0);

        assert_eq!(page.groups[0].episode.episode_number, 400.0);
    }

    #[test]
    fn test_title_match_without_moments_is_included() {
        let page = search(&index(), "redemption", SearchSort::Newest, 0);

        assert_eq!(page.total_episodes, 1);
        assert_eq!(page.total_moments, 0);
        assert!(page.groups[0].hits.is_empty());
    }

    #[test]
    fn test_fractional_episode_numbers_group_correctly() {
        let page = search(&index(), "confederate", SearchSort::Newest, 0);

        assert_eq!(page.groups[0].episode.episode_number, 524.1);
        assert_eq!(page.groups[0].match_count, 1);
    }

    #[test]
    fn test_pages_through_groups() {
        let events = (0..30)
            .map(|n| event(n as f32, 0, "kyle"))
            .collect::<Vec<_>>();
        let episodes = (0..30)
            .rev()
            .map(|n| episode(n as f32, "x"))
            .collect::<Vec<_>>();
        let index = SearchIndex {
            episodes: episodes.into_boxed_slice(),
            events: events.into_boxed_slice(),
        };

        let first = search(&index, "kyle", SearchSort::Newest, 0);
        let last = search(&index, "kyle", SearchSort::Newest, 2);

        assert_eq!(first.groups.len(), GROUPS_PER_PAGE);
        assert_eq!(first.next_page, Some(1));
        assert_eq!(last.groups.len(), 30 - 2 * GROUPS_PER_PAGE);
        assert_eq!(last.next_page, None);
    }
}
