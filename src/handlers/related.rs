use std::collections::HashMap;

use crate::extractors::AppQuery;
use crate::models::errors::ErrorResponseBody;
use crate::models::search::{RelatedMoment, RelatedParams};
use crate::models::success_response::SuccessResponse;
use crate::search::{episode_key, SEARCH_INDEX};

const LIMIT: usize = 6;

#[utoipa::path(
    get,
    path = "/api/v1/events/related",
    params(RelatedParams),
    responses(
        (status = 200, description = "Moments in other episodes about the same thing, strongest first", body = SuccessResponse<Vec<RelatedMoment>>),
        (status = 400, description = "Invalid request", body = ErrorResponseBody)
    ),
    tag = "Events"
)]
pub async fn related_pka_events(
    AppQuery(params): AppQuery<RelatedParams>,
) -> SuccessResponse<Vec<RelatedMoment>> {
    let index = SEARCH_INDEX.read().await;
    let key = episode_key(params.episode);

    // An unknown moment simply has no related moments.
    let Some(source) = index.events.iter().position(|event| {
        episode_key(event.episode_number()) == key && event.timestamp() == params.timestamp
    }) else {
        return SuccessResponse::new(Vec::new());
    };

    let episodes = index
        .episodes
        .iter()
        .map(|episode| (episode_key(episode.episode_number), episode))
        .collect::<HashMap<_, _>>();

    let related = index
        .related
        .related(&index.events, source, LIMIT)
        .into_iter()
        .filter_map(|(candidate, shared_terms)| {
            let event = &index.events[candidate];
            let episode = episodes.get(&episode_key(event.episode_number()))?;

            Some(RelatedMoment {
                episode: (*episode).clone(),
                timestamp: event.timestamp(),
                description: event.description.clone(),
                length_seconds: event.length_seconds(),
                shared_terms,
            })
        })
        .collect();

    SuccessResponse::new(related)
}
