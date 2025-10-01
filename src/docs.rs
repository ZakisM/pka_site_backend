use serde::Serialize;
use utoipa::openapi::OpenApi;
use utoipa::OpenApi as OpenApiTrait;
use utoipa::ToSchema;

use crate::handlers::{episode, event, search, static_files};
use crate::models::errors::ErrorResponseBody;
use crate::models::pka_episode::PkaEpisode;
use crate::models::pka_episode_with_all::PkaEpisodeWithAll;
use crate::models::pka_event::PkaEvent;
use crate::models::pka_youtube_details::PkaYoutubeDetails;
use crate::models::search::{PkaEventSearchResult, SearchQuery};

#[derive(Serialize, ToSchema)]
pub struct EpisodeResponse {
    pub code: u16,
    pub data: PkaEpisodeWithAll,
}

#[derive(Serialize, ToSchema)]
pub struct YoutubeLinkResponse {
    pub code: u16,
    pub data: String,
}

#[derive(Serialize, ToSchema)]
pub struct EventResponse {
    pub code: u16,
    pub data: PkaEventSearchResult,
}

#[derive(OpenApiTrait)]
#[openapi(
    info(title = "PKA Index API", version = "1.0"),
    paths(
        episode::watch_pka_episode,
        episode::find_pka_episode_youtube_link,
        episode::latest_pka_episode,
        episode::random_pka_episode,
        event::random_pka_event,
        search::search_pka_episode,
        search::search_pka_event,
        static_files::robots_txt,
        static_files::sitemap_xml
    ),
    components(schemas(
        EpisodeResponse,
        YoutubeLinkResponse,
        EventResponse,
        ErrorResponseBody,
        PkaEpisode,
        PkaEvent,
        PkaEpisodeWithAll,
        PkaYoutubeDetails,
        PkaEventSearchResult,
        SearchQuery
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
