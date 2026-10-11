//! "In other episodes": moments elsewhere in the archive about the same thing
//! As a given moment, matched on the distinctive words in their descriptions.
//!
//! Words are weighted by how rare they are across every moment (IDF), so a
//! Shared "Hellraiser" counts for far more than a shared "movie", and words
//! Common enough to say nothing about the topic — the hosts' names, "story" —
//! Are ignored outright.

use std::collections::HashMap;

use compact_str::CompactString;

use crate::models::pka_event::PkaEvent;

// A word in more than this share of moments says nothing about the topic.
const MAX_DOCUMENT_SHARE: f32 = 0.005;
// Roughly one shared word that appears in ~30 moments, or a few less rare ones.
const MIN_SCORE_DOCUMENTS: f32 = 30.0;
// A lone shared name must be this rare: "DMT" (19 moments) is a subject,
// "Larry" (27, several different Larrys) is not.
const MAX_SINGLE_NAME_DOCUMENTS: usize = 20;

const STOPWORDS: &[&str] = &[
    "about",
    "after",
    "again",
    "against",
    "all",
    "and",
    "any",
    "are",
    "back",
    "been",
    "before",
    "being",
    "best",
    "between",
    "both",
    "but",
    "can",
    "could",
    "did",
    "does",
    "doing",
    "don't",
    "down",
    "during",
    "each",
    "even",
    "ever",
    "every",
    "few",
    "for",
    "from",
    "get",
    "gets",
    "getting",
    "got",
    "had",
    "has",
    "have",
    "having",
    "her",
    "here",
    "him",
    "his",
    "how",
    "into",
    "its",
    "just",
    "like",
    "made",
    "make",
    "makes",
    "many",
    "more",
    "most",
    "much",
    "new",
    "not",
    "now",
    "off",
    "old",
    "once",
    "one",
    "only",
    "other",
    "our",
    "out",
    "over",
    "own",
    "really",
    "same",
    "she",
    "should",
    "some",
    "still",
    "such",
    "than",
    "that",
    "the",
    "their",
    "them",
    "then",
    "there",
    "these",
    "they",
    "thing",
    "things",
    "this",
    "those",
    "through",
    "too",
    "two",
    "under",
    "until",
    "very",
    "was",
    "way",
    "were",
    "what",
    "when",
    "where",
    "which",
    "while",
    "who",
    "why",
    "will",
    "with",
    "would",
    "you",
    "your",
    // Filler that pairs unrelated moments ("DIY gone wrong" / "pet ducks gone
    // Wrong"): framing words, common verbs and adjectives.
    "actually",
    "bad",
    "big",
    "came",
    "come",
    "comes",
    "day",
    "discuss",
    "experience",
    "finally",
    "find",
    "first",
    "found",
    "investigate",
    "investigates",
    "give",
    "goes",
    "going",
    "gone",
    "good",
    "great",
    "guy",
    "guys",
    "know",
    "last",
    "life",
    "little",
    "look",
    "man",
    "need",
    "news",
    "people",
    "react",
    "reacts",
    "real",
    "said",
    "say",
    "says",
    "show",
    "story",
    "stories",
    "take",
    "talk",
    "talks",
    "tell",
    "think",
    "time",
    "update",
    "updates",
    "want",
    "week",
    "world",
    "wrong",
    "year",
    "years",
];

// Above this share of capitalised words a description is in Title Case, and
// Capitals no longer say which words are names.
const TITLE_CASE_SHARE: f32 = 0.6;

fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !(c.is_alphanumeric() || c == '\'' || c == '’'))
        .map(|raw| raw.trim_matches(|c| c == '\'' || c == '’'))
        .filter(|raw| !raw.is_empty())
}

/// A word in comparable form: lowercase, possessives and plurals folded
/// ("Woody's" → "woody", "movies" → "movie"); `None` for short words, numbers
/// And stopwords.
fn normalise(raw: &str) -> Option<CompactString> {
    let lower = raw.to_lowercase();
    let word = lower
        .strip_suffix("'s")
        .or_else(|| lower.strip_suffix("’s"))
        .unwrap_or(&lower);

    if word.chars().count() < 3
        || word.chars().all(|c| c.is_ascii_digit())
        || STOPWORDS.contains(&word)
    {
        return None;
    }

    let folded = if word.len() > 4 && word.ends_with('s') && !word.ends_with("ss") {
        &word[..word.len() - 1]
    } else {
        word
    };

    Some(CompactString::from(folded))
}

/// The distinct comparable words of a description.
pub fn terms(text: &str) -> Vec<CompactString> {
    let mut out = words(text).filter_map(normalise).collect::<Vec<_>>();

    out.sort_unstable();
    out.dedup();
    out
}

/// Words written as names: capitalised somewhere other than the start. Empty
/// For Title Case descriptions, where capitals carry no signal.
fn names(text: &str) -> Vec<CompactString> {
    let candidates = words(text)
        .skip(1)
        .filter(|word| word.chars().count() >= 3 && word.chars().any(char::is_alphabetic))
        .collect::<Vec<_>>();
    let capitalised = candidates
        .iter()
        .filter(|word| word.chars().next().is_some_and(char::is_uppercase))
        .collect::<Vec<_>>();

    if candidates.is_empty()
        || capitalised.len() as f32 / candidates.len() as f32 > TITLE_CASE_SHARE
    {
        return Vec::new();
    }

    capitalised
        .into_iter()
        .filter_map(|word| normalise(word))
        .collect()
}

/// Word statistics over every moment, rebuilt with the search index.
#[derive(Default)]
pub struct RelatedIndex {
    idf: HashMap<CompactString, f32>,
    // Word → indices of the moments using it, for words rare enough to count.
    postings: HashMap<CompactString, Vec<usize>>,
    min_score: f32,
}

impl RelatedIndex {
    pub fn build(events: &[PkaEvent]) -> Self {
        let total = events.len().max(1) as f32;
        let max_documents = (total * MAX_DOCUMENT_SHARE).max(20.0) as usize;
        let mut postings: HashMap<CompactString, Vec<usize>> = HashMap::new();

        for (index, event) in events.iter().enumerate() {
            for term in terms(event.description()) {
                postings.entry(term).or_default().push(index);
            }
        }

        postings.retain(|_, moments| moments.len() <= max_documents);

        let idf = postings
            .iter()
            .map(|(term, moments)| (term.clone(), (total / moments.len() as f32).ln()))
            .collect();

        Self {
            idf,
            postings,
            min_score: (total / MIN_SCORE_DOCUMENTS).ln(),
        }
    }

    /// The best-matching moment from each other episode, strongest first, with
    /// The words it shares with `events[source]`.
    pub fn related(
        &self,
        events: &[PkaEvent],
        source: usize,
        limit: usize,
    ) -> Vec<(usize, Vec<CompactString>)> {
        let Some(source_event) = events.get(source) else {
            return Vec::new();
        };

        let source_episode = source_event.episode_number();
        let source_names = names(source_event.description());
        let mut scores: HashMap<usize, (f32, Vec<CompactString>)> = HashMap::new();

        for term in terms(source_event.description()) {
            let (Some(moments), Some(&weight)) = (self.postings.get(&term), self.idf.get(&term))
            else {
                continue;
            };

            for &candidate in moments {
                if events[candidate].episode_number() == source_episode {
                    continue;
                }

                let entry = scores.entry(candidate).or_default();

                entry.0 += weight;
                entry.1.push(term.clone());
            }
        }

        // One moment per episode: the strongest.
        let mut best_per_episode: HashMap<i64, (usize, f32, Vec<CompactString>)> = HashMap::new();

        for (candidate, (score, shared)) in scores {
            if score < self.min_score {
                continue;
            }

            // One shared ordinary word ("corrupt", "axe") is a coincidence; one
            // Shared name ("DMT", "Hellraiser") is the same subject.
            if let [only] = shared.as_slice() {
                let rare = self
                    .postings
                    .get(only)
                    .is_some_and(|moments| moments.len() <= MAX_SINGLE_NAME_DOCUMENTS);
                let named_in_both = source_names.contains(only)
                    && names(events[candidate].description()).contains(only);

                if !rare || !named_in_both {
                    continue;
                }
            }

            let key = (f64::from(events[candidate].episode_number()) * 10.0).round() as i64;
            let replace = best_per_episode
                .get(&key)
                .is_none_or(|(_, best, _)| score > *best);

            if replace {
                best_per_episode.insert(key, (candidate, score, shared));
            }
        }

        let mut ranked = best_per_episode.into_values().collect::<Vec<_>>();

        ranked.sort_by(|a, b| {
            b.1.total_cmp(&a.1).then_with(|| {
                events[b.0]
                    .episode_number()
                    .total_cmp(&events[a.0].episode_number())
            })
        });

        ranked
            .into_iter()
            .take(limit)
            .map(|(candidate, _, shared)| (candidate, shared))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_terms_fold_possessives_plurals_and_stopwords() {
        assert_eq!(
            terms("Why Woody's movies are the BEST in 2024"),
            vec!["movie", "woody"]
        );
    }

    #[test]
    fn test_names_skip_the_first_word_and_title_case() {
        assert_eq!(
            names("Why Joe Rogan is wrong about DMT"),
            vec!["joe", "rogan", "dmt"]
        );
        assert!(names("Why Kyle Loves Spawn And Sons Of Anarchy").is_empty());
    }

    #[test]
    fn test_a_single_shared_ordinary_word_is_not_enough() {
        let mut events = vec![
            event(612.0, 10, "Kyle and Larry talk about corrupt guards"),
            event(732.0, 20, "Conor and why some churches are corrupt"),
            event(461.0, 30, "Just how good is DMT really"),
            event(813.0, 40, "Why Kyle wants to try DMT"),
        ];

        for n in 0..200 {
            events.push(event(n as f32, n, &format!("filler topic number{n}")));
        }

        let index = RelatedIndex::build(&events);

        assert!(index.related(&events, 0, 6).is_empty());
        assert_eq!(index.related(&events, 2, 6).len(), 1);
    }

    #[test]
    fn test_related_matches_rare_words_across_episodes_only() {
        let mut events = vec![
            event(825.0, 10, "Why Pinhead from Hellraiser is terrifying"),
            event(825.0, 20, "More Hellraiser talk in the same episode"),
            event(
                818.0,
                30,
                "Roger Rabbit, Hellraiser & Brave Little Toaster: Movies that terrified the boys as kids",
            ),
            event(461.0, 40, "Dead girl, Hellraiser and Oz"),
            event(400.0, 50, "Taylor's bible story"),
        ];

        // Pad with unrelated moments so the shared words count as rare.
        for n in 0..200 {
            events.push(event(n as f32, n, &format!("filler topic number{n}")));
        }

        let index = RelatedIndex::build(&events);
        let related = index.related(&events, 0, 6);
        let episodes = related
            .iter()
            .map(|(i, _)| events[*i].episode_number())
            .collect::<Vec<_>>();

        assert_eq!(episodes, vec![818.0, 461.0]);
        assert_eq!(related[0].1, vec![CompactString::from("hellraiser")]);
    }
}
