use utoipa::OpenApi as OpenApiTrait;
use utoipa::openapi::OpenApi;

use crate::handlers::{episode, event, related, search, static_files};
use crate::models::errors::ErrorResponseBody;
use crate::models::pka_episode::PkaEpisode;
use crate::models::pka_episode_with_all::PkaEpisodeWithAll;
use crate::models::pka_event::PkaEvent;
use crate::models::pka_event::RandomPkaEvent;
use crate::models::pka_youtube_details::PkaYoutubeDetails;
use crate::models::search::{EpisodeSummary, RelatedMoment, SearchPage, SearchSort};
use crate::models::success_response::SuccessResponse;

#[derive(OpenApiTrait)]
#[openapi(
    info(title = "PKA Index API", version = "1.0"),
    paths(
        episode::all_pka_episodes,
        episode::watch_pka_episode,
        episode::find_pka_episode_youtube_link,
        episode::latest_pka_episode,
        episode::random_pka_episode,
        event::random_pka_event,
        related::related_pka_events,
        search::search_pka,
        static_files::robots_txt,
        static_files::sitemap_xml
    ),
    components(schemas(
        SuccessResponse<PkaEpisodeWithAll>,
        SuccessResponse<String>,
        SuccessResponse<RandomPkaEvent>,
        SuccessResponse<SearchPage>,
        SuccessResponse<Vec<EpisodeSummary>>,
        SuccessResponse<Vec<RelatedMoment>>,
        ErrorResponseBody,
        PkaEpisode,
        PkaEvent,
        PkaYoutubeDetails,
        SearchSort
    )),
    tags(
        (name = "Episodes"),
        (name = "Events"),
        (name = "Search"),
        (name = "Static")
    )
)]
pub struct ApiDoc;

pub fn openapi() -> OpenApi {
    ApiDoc::openapi()
}
