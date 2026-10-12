//! Finding moments: every moment, and every episode title, containing all the
//! words of a query, grouped by episode and paged.

use std::cmp::Reverse;

use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};

use crate::catalog::Catalog;
use crate::models::search::{SearchGroup, SearchHit, SearchPage, SearchSort};

pub const GROUPS_PER_PAGE: usize = 12;
// Beyond this a query is a paste, not a search; it also bounds the bitmask.
const MAX_TERMS: usize = 16;

/// Matches text containing every word of the query, case-insensitively.
struct Matcher {
    automaton: AhoCorasick,
    // Which query word each pattern stands for: a word can have several
    // Spellings (see `spellings`).
    pattern_terms: Vec<u32>,
    all_seen: u32,
}

/// The ways a query word may be written in the archive. Phones and Macs type
/// Curly quotes by default while most descriptions use straight ones (and a
/// Few titles the reverse), so "kyle’s" must find "Kyle's" and vice versa.
fn spellings(term: &str) -> Vec<String> {
    let straight = term.replace(['‘', '’'], "'").replace(['“', '”'], "\"");
    let mut out = vec![straight.clone()];

    if straight.contains('\'') {
        out.push(straight.replace('\'', "’"));
    }

    if straight.contains('"') {
        out.push(straight.replace('"', "“"));
        out.push(straight.replace('"', "”"));
    }

    out
}

impl Matcher {
    fn new(query: &str) -> Option<Self> {
        let mut terms = query
            .split_whitespace()
            .map(|word| spellings(&word.to_lowercase()).swap_remove(0))
            .collect::<Vec<_>>();

        // A repeated word would leave its second bit unset forever.
        terms.sort_unstable();
        terms.dedup();
        terms.truncate(MAX_TERMS);

        if terms.is_empty() {
            return None;
        }

        let mut patterns = Vec::new();
        let mut pattern_terms = Vec::new();

        for (index, term) in terms.iter().enumerate() {
            for spelling in spellings(term) {
                patterns.push(spelling);
                pattern_terms.push(index as u32);
            }
        }

        // Overlapping, so "tay taylor" sees both words in "Taylor".
        let automaton = AhoCorasickBuilder::new()
            .ascii_case_insensitive(true)
            .match_kind(MatchKind::Standard)
            .build(&patterns)
            .ok()?;

        Some(Self {
            automaton,
            pattern_terms,
            all_seen: (1 << terms.len()) - 1,
        })
    }

    fn matches(&self, text: &str) -> bool {
        let mut seen = 0_u32;

        for found in self.automaton.find_overlapping_iter(text) {
            seen |= 1 << self.pattern_terms[found.pattern().as_usize()];

            if seen == self.all_seen {
                return true;
            }
        }

        false
    }
}

/// One page of results, grouped by episode: every episode with a matching
/// Moment, plus any whose title matches.
pub fn search(catalog: &Catalog, query: &str, sort: SearchSort, page: usize) -> SearchPage {
    let Some(matcher) = Matcher::new(query) else {
        return SearchPage::default();
    };

    // One plain pass, episode by episode, so groups come out newest first and
    // Their moments in timestamp order. The server has a single core:
    // Splitting the scan across threads would only add overhead.
    let mut groups = catalog
        .episodes
        .iter()
        .filter_map(|episode| {
            let hits = catalog
                .events_of(episode)
                .iter()
                .filter(|event| matcher.matches(event.description()))
                .collect::<Vec<_>>();

            (!hits.is_empty() || matcher.matches(&episode.summary.title)).then_some((episode, hits))
        })
        .collect::<Vec<_>>();

    if sort == SearchSort::Matches {
        // Stable, so equal counts keep newest-first order.
        groups.sort_by_key(|(_, hits)| Reverse(hits.len()));
    }

    let total_moments = groups.iter().map(|(_, hits)| hits.len()).sum();
    let total_episodes = groups.len();
    let start = page.saturating_mul(GROUPS_PER_PAGE);

    let groups = groups
        .into_iter()
        .skip(start)
        .take(GROUPS_PER_PAGE)
        .map(|(episode, hits)| SearchGroup {
            episode: episode.summary.clone(),
            match_count: hits.len(),
            // Every match: someone searching reads them to find the one they
            // Mean, and a page of twelve episodes stays small however broad.
            hits: hits.into_iter().map(SearchHit::from).collect(),
        })
        .collect();

    SearchPage {
        total_moments,
        total_episodes,
        groups,
        // Saturating: a hand-written page number near usize::MAX must not wrap.
        next_page: (start.saturating_add(GROUPS_PER_PAGE) < total_episodes)
            .then_some(page.saturating_add(1)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::test_data::{catalog, episode, event};

    fn index() -> Catalog {
        catalog(
            vec![
                episode(825.0, "PKA 825 W/ Harley: Redemption Of Wings"),
                episode(524.1, "Part 1 Confederate Talk"),
                episode(400.0, "PKA 400 w/Tucker - Taylor's Bible Story"),
            ],
            vec![
                event(825.0, 1547, "Why Pinhead from Hellraiser is terrifying"),
                event(825.0, 30, "Taylor reads the sponsors"),
                event(400.0, 900, "Taylor's bible story"),
                event(400.0, 100, "Taylor and Kyle argue"),
                event(400.0, 50, "Taylor intro"),
                event(400.0, 5000, "Taylor outro"),
                event(524.1, 10, "Confederate talk part one"),
            ],
        )
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
        assert!(
            Matcher::new("tay taylor")
                .unwrap()
                .matches("Taylor's bible story")
        );
    }

    #[test]
    fn test_curly_and_straight_apostrophes_match_each_other() {
        assert!(Matcher::new("kyle’s").unwrap().matches("Kyle's new bike"));
        assert!(Matcher::new("kyle's").unwrap().matches("Kyle’s new bike"));
        assert!(!Matcher::new("kyle’s").unwrap().matches("Kyles new bike"));
    }

    #[test]
    fn test_huge_page_number_is_empty_not_a_panic() {
        let page = search(&index(), "taylor", SearchSort::Newest, usize::MAX);

        assert!(page.groups.is_empty());
        assert_eq!(page.next_page, None);
    }

    #[test]
    fn test_blank_query_returns_nothing() {
        let page = search(&index(), "   ", SearchSort::Newest, 0);

        assert_eq!(page.total_episodes, 0);
        assert!(page.groups.is_empty());
    }

    #[test]
    fn test_groups_by_episode_newest_first_with_every_hit_in_order() {
        let page = search(&index(), "taylor", SearchSort::Newest, 0);

        assert_eq!(page.total_moments, 5);
        assert_eq!(page.total_episodes, 2);
        assert_eq!(page.groups[0].episode.episode_number, 825.0);

        let older = &page.groups[1];

        assert_eq!(older.match_count, 4);
        assert_eq!(
            older
                .hits
                .iter()
                .map(|hit| hit.timestamp)
                .collect::<Vec<_>>(),
            vec![50, 100, 900, 5000]
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
        let episodes = (0..30).map(|n| episode(n as f32, "x")).collect::<Vec<_>>();
        let index = catalog(episodes, events);

        let first = search(&index, "kyle", SearchSort::Newest, 0);
        let last = search(&index, "kyle", SearchSort::Newest, 2);

        assert_eq!(first.groups.len(), GROUPS_PER_PAGE);
        assert_eq!(first.next_page, Some(1));
        assert_eq!(last.groups.len(), 30 - 2 * GROUPS_PER_PAGE);
        assert_eq!(last.next_page, None);
    }
}
