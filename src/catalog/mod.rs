//! Everything the API serves, read from SQLite into memory in one go and
//! swapped wholesale when the data changes. Requests never touch the database,
//! and never see an episode from one snapshot with moments from another.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::Range;
use std::sync::{Arc, LazyLock, PoisonError, RwLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::Context;
use axum::body::Bytes;
use rand::seq::IndexedRandom;
use tracing::{info, warn};

use crate::Repo;
use crate::conduit::sqlite::{pka_episode, pka_event, pka_youtube_details};
use crate::models::pka_episode::PkaEpisode;
use crate::models::pka_episode_with_all::PkaEpisodeWithAll;
use crate::models::pka_event::PkaEvent;
use crate::models::pka_youtube_details::PkaYoutubeDetails;
use crate::models::search::EpisodeSummary;
use crate::models::success_response::SuccessResponse;

mod encoded;
pub mod related;
pub mod search;
mod sitemap;

pub use encoded::Encoded;
use related::RelatedIndex;

pub struct CatalogEpisode {
    pub summary: EpisodeSummary,
    pub episode: PkaEpisode,
    pub youtube_details: PkaYoutubeDetails,
    /// Where its moments sit in `Catalog::events`.
    pub events: Range<usize>,
}

#[derive(Default)]
pub struct Catalog {
    /// Newest first: the order of the episode list and of search results.
    pub episodes: Box<[CatalogEpisode]>,
    /// Grouped by episode in the same order, each episode's in timestamp order.
    pub events: Box<[PkaEvent]>,
    pub related: RelatedIndex,
    /// Indices of the moments worth landing on at random: not the show's
    /// Bookends or ad reads.
    pub random_pool: Box<[u32]>,
    /// The episode list and the sitemap are the same for every request, so
    /// They are rendered once per snapshot rather than once per request.
    pub episodes_json: Encoded,
    pub sitemap_xml: Bytes,
    /// Changes whenever anything the API answers with could: the ETag of
    /// every answer drawn from this snapshot.
    pub version: u64,
}

// When this process started. Part of every version, so a deploy that changes
// How the same data is rendered can't be answered "not modified" from a
// Browser's copy in the old shape.
static STARTED: LazyLock<u128> = LazyLock::new(|| {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos())
});

fn fingerprint(episodes: &[CatalogEpisode], list: &[u8], events: &[PkaEvent]) -> u64 {
    let mut hasher = DefaultHasher::new();

    STARTED.hash(&mut hasher);
    list.hash(&mut hasher);

    // The list covers titles, dates and counts; episode pages also show these.
    for episode in episodes {
        episode.episode.name.hash(&mut hasher);
    }

    for event in events {
        event.episode_number().to_bits().hash(&mut hasher);
        event.timestamp().hash(&mut hasher);
        event.description().hash(&mut hasher);
        event.length_seconds().hash(&mut hasher);
    }

    hasher.finish()
}

// Episode numbers can be fractional (524.1); compare them as tenths.
pub fn episode_key(number: f32) -> i64 {
    (f64::from(number) * 10.0).round() as i64
}

// The show's bookends and sponsor reads: poor picks at random, and they say
// Nothing about a topic.
const FILLER: [&[u8]; 3] = [b"intro", b"outro", b"ad read"];

pub fn is_filler(description: &str) -> bool {
    FILLER.iter().any(|word| {
        description
            .as_bytes()
            .windows(word.len())
            .any(|window| window.eq_ignore_ascii_case(word))
    })
}

fn random_pool(events: &[PkaEvent]) -> Box<[u32]> {
    events
        .iter()
        .enumerate()
        .filter(|(_, event)| !is_filler(event.description()))
        .map(|(index, _)| index as u32)
        .collect()
}

impl Catalog {
    pub fn build(
        mut episodes: Vec<PkaEpisode>,
        details: Vec<PkaYoutubeDetails>,
        events: Vec<PkaEvent>,
    ) -> anyhow::Result<Self> {
        let mut details = details
            .into_iter()
            .map(|details| (episode_key(details.episode_number), details))
            .collect::<HashMap<_, _>>();
        let mut events_by_episode: HashMap<i64, Vec<PkaEvent>> = HashMap::new();

        for event in events {
            events_by_episode
                .entry(episode_key(event.episode_number()))
                .or_default()
                .push(event);
        }

        episodes.sort_unstable_by(|a, b| b.number.total_cmp(&a.number));

        let mut catalog_episodes = Vec::with_capacity(episodes.len());
        let mut all_events = Vec::new();

        for episode in episodes {
            let key = episode_key(episode.number);
            let Some(youtube_details) = details.remove(&key) else {
                warn!(
                    "Episode {} has no YouTube details, so it can't be played; leaving it out",
                    episode.number
                );
                continue;
            };
            let mut moments = events_by_episode.remove(&key).unwrap_or_default();

            moments.sort_unstable_by_key(PkaEvent::timestamp);

            let start = all_events.len();

            all_events.extend(moments);

            catalog_episodes.push(CatalogEpisode {
                summary: EpisodeSummary {
                    episode_number: episode.number,
                    upload_date: episode.upload_date,
                    title: youtube_details.title.clone(),
                    length_seconds: youtube_details.length_seconds,
                    video_id: youtube_details.video_id.clone(),
                    event_count: (all_events.len() - start) as i32,
                },
                episode,
                youtube_details,
                events: start..all_events.len(),
            });
        }

        let summaries = catalog_episodes
            .iter()
            .map(|episode| &episode.summary)
            .collect::<Vec<_>>();
        let episodes_json = serde_json::to_vec(&SuccessResponse::new(summaries))
            .context("Failed to render the episode list")?;
        let sitemap_xml = sitemap::render(&catalog_episodes)?;
        let version = fingerprint(&catalog_episodes, &episodes_json, &all_events);

        Ok(Self {
            related: RelatedIndex::build(&all_events),
            random_pool: random_pool(&all_events),
            episodes: catalog_episodes.into_boxed_slice(),
            events: all_events.into_boxed_slice(),
            episodes_json: Encoded::new(episodes_json)
                .context("Failed to compress the episode list")?,
            sitemap_xml: sitemap_xml.into(),
            version,
        })
    }

    pub fn episode(&self, number: f32) -> Option<&CatalogEpisode> {
        let key = episode_key(number);

        // Newest first, so a larger number sorts earlier.
        self.episodes
            .binary_search_by(|episode| key.cmp(&episode_key(episode.episode.number)))
            .ok()
            .map(|index| &self.episodes[index])
    }

    pub fn latest(&self) -> Option<&CatalogEpisode> {
        self.episodes.first()
    }

    pub fn random_episode(&self) -> Option<&CatalogEpisode> {
        self.episodes.choose(&mut rand::rng())
    }

    pub fn events_of(&self, episode: &CatalogEpisode) -> &[PkaEvent] {
        &self.events[episode.events.clone()]
    }

    /// The episode a moment (by its index in `events`) belongs to.
    pub fn episode_of(&self, event: usize) -> &CatalogEpisode {
        &self.episodes[self
            .episodes
            .partition_point(|episode| episode.events.end <= event)]
    }

    /// The index in `events` of an episode's moment starting at `timestamp`.
    pub fn find_event(&self, number: f32, timestamp: i32) -> Option<usize> {
        let episode = self.episode(number)?;
        let offset = self
            .events_of(episode)
            .binary_search_by_key(&timestamp, PkaEvent::timestamp)
            .ok()?;

        Some(episode.events.start + offset)
    }

    pub fn with_all(&self, episode: &CatalogEpisode) -> PkaEpisodeWithAll {
        PkaEpisodeWithAll::new(
            episode.episode.clone(),
            episode.youtube_details.clone(),
            self.events_of(episode).to_vec(),
        )
    }
}

/// The current snapshot, shared by every request and replaced on refresh.
/// Readers take a cheap `Arc` clone and never wait on a rebuild.
#[derive(Clone, Default)]
pub struct SharedCatalog(Arc<RwLock<Arc<Catalog>>>);

impl SharedCatalog {
    pub fn new(catalog: Catalog) -> Self {
        Self(Arc::new(RwLock::new(Arc::new(catalog))))
    }

    pub fn get(&self) -> Arc<Catalog> {
        self.0
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn replace(&self, catalog: Catalog) {
        *self.0.write().unwrap_or_else(PoisonError::into_inner) = Arc::new(catalog);
    }
}

pub async fn load(repo: &Repo) -> anyhow::Result<Catalog> {
    let started = Instant::now();

    // One read transaction, so a new episode and its moments arrive together.
    let mut tx = repo
        .begin()
        .await
        .context("Failed to start the catalog read")?;
    let episodes = pka_episode::all(&mut *tx)
        .await
        .context("Failed to load episodes")?;
    let details = pka_youtube_details::all(&mut *tx)
        .await
        .context("Failed to load YouTube details")?;
    let events = pka_event::all(&mut *tx)
        .await
        .context("Failed to load moments")?;

    tx.commit()
        .await
        .context("Failed to finish the catalog read")?;

    // A few hundred milliseconds of CPU: off the async workers, so requests
    // Keep being answered from the current snapshot meanwhile.
    let catalog = tokio::task::spawn_blocking(move || Catalog::build(episodes, details, events))
        .await
        .context("Building the catalog panicked")??;

    info!(
        episodes = catalog.episodes.len(),
        moments = catalog.events.len(),
        elapsed_ms = started.elapsed().as_millis() as u64,
        "Catalog loaded"
    );

    Ok(catalog)
}

pub async fn refresh(repo: &Repo, catalog: &SharedCatalog) -> anyhow::Result<()> {
    catalog.replace(load(repo).await?);

    Ok(())
}

#[cfg(test)]
pub mod test_data {
    use super::*;

    pub fn episode(number: f32, title: &str) -> (PkaEpisode, PkaYoutubeDetails) {
        (
            PkaEpisode::new(
                number,
                format!("PKA {number}").into(),
                format!("video{number}").into(),
                1_700_000_000 + (number * 86_400.0) as i64,
            ),
            PkaYoutubeDetails::new(
                format!("video{number}").into(),
                number,
                title.into(),
                14_400,
            ),
        )
    }

    pub fn event(number: f32, timestamp: i32, description: &str) -> PkaEvent {
        PkaEvent::new(
            format!("{number}-{timestamp}").into(),
            number,
            timestamp,
            description.into(),
            60,
            0,
        )
    }

    pub fn catalog(
        episodes: Vec<(PkaEpisode, PkaYoutubeDetails)>,
        events: Vec<PkaEvent>,
    ) -> Catalog {
        let (episodes, details) = episodes.into_iter().unzip();

        Catalog::build(episodes, details, events).expect("Failed to build the catalog")
    }
}

#[cfg(test)]
mod tests {
    use super::test_data::{catalog, episode, event};
    use super::*;

    fn sample() -> Catalog {
        catalog(
            vec![
                episode(400.0, "PKA 400"),
                episode(524.1, "Part 1"),
                episode(825.0, "PKA 825"),
            ],
            vec![
                event(524.1, 10, "Confederate talk"),
                event(825.0, 90, "Second"),
                event(825.0, 30, "First"),
                event(400.0, 5, "Oldest"),
                // An orphan, whose episode isn't in the index, is dropped.
                event(1.0, 0, "Nowhere"),
            ],
        )
    }

    #[test]
    fn test_episodes_newest_first_with_their_moments_in_order() {
        let catalog = sample();
        let numbers = catalog
            .episodes
            .iter()
            .map(|episode| episode.episode.number)
            .collect::<Vec<_>>();

        assert_eq!(numbers, vec![825.0, 524.1, 400.0]);
        assert_eq!(catalog.events.len(), 4);

        let newest = catalog.latest().unwrap();
        let descriptions = catalog
            .events_of(newest)
            .iter()
            .map(PkaEvent::description)
            .collect::<Vec<_>>();

        assert_eq!(descriptions, vec!["First", "Second"]);
        assert_eq!(newest.summary.event_count, 2);
    }

    #[test]
    fn test_lookups_by_number_moment_and_index() {
        let catalog = sample();

        assert_eq!(catalog.episode(524.1).unwrap().summary.title, "Part 1");
        assert!(catalog.episode(524.0).is_none());

        let moment = catalog.find_event(825.0, 90).unwrap();

        assert_eq!(catalog.events[moment].description(), "Second");
        assert_eq!(catalog.episode_of(moment).episode.number, 825.0);
        assert_eq!(catalog.episode_of(3).episode.number, 400.0);
        assert!(catalog.find_event(825.0, 91).is_none());
    }

    #[test]
    fn test_episode_without_youtube_details_is_left_out() {
        let (orphan, _) = episode(9.0, "No video");
        let (kept, details) = episode(10.0, "Has video");
        let catalog = Catalog::build(vec![orphan, kept], vec![details], Vec::new()).unwrap();

        assert_eq!(catalog.episodes.len(), 1);
        assert!(catalog.episode(9.0).is_none());
    }

    #[test]
    fn test_random_pool_skips_bookends_and_ad_reads() {
        let events = vec![
            event(1.0, 0, "Woody introduces the show (INTRO)"),
            event(1.0, 10, "Ad read: Harry's"),
            event(1.0, 20, "Kyle's prison stories"),
            event(1.0, 30, "Outro"),
        ];

        assert_eq!(&*random_pool(&events), &[2]);
    }
}
