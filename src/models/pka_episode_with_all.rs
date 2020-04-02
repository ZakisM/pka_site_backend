use serde::Serialize;

use crate::models::pka_episode::PkaEpisode;
use crate::models::pka_event::PkaEvent;
use crate::models::pka_youtube_details::PkaYoutubeDetails;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PkaEpisodeWithAll {
    episode: PkaEpisode,
    youtube_details: PkaYoutubeDetails,
    events: Vec<PkaEvent>,
}

impl PkaEpisodeWithAll {
    pub fn new(
        episode: PkaEpisode,
        youtube_details: PkaYoutubeDetails,
        events: Vec<PkaEvent>,
    ) -> Self {
        PkaEpisodeWithAll {
            episode,
            youtube_details,
            events,
        }
    }
}
