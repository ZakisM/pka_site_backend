use axum::extract::State;

use crate::app_state::AppState;
use crate::extractors::AppQuery;
use crate::models::errors::ErrorResponseBody;
use crate::models::search::{RelatedMoment, RelatedParams};
use crate::models::success_response::SuccessResponse;

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
    State(state): State<AppState>,
) -> SuccessResponse<Vec<RelatedMoment>> {
    let catalog = state.catalog.get();

    // An unknown moment simply has no related moments.
    let Some(source) = catalog.find_event(params.episode, params.timestamp) else {
        return SuccessResponse::new(Vec::new());
    };

    let related = catalog
        .related
        .related(&catalog.events, source, LIMIT)
        .into_iter()
        .map(|(candidate, shared_terms)| {
            let event = &catalog.events[candidate];

            RelatedMoment {
                episode: catalog.episode_of(candidate).summary.clone(),
                timestamp: event.timestamp(),
                description: event.description.clone(),
                length_seconds: event.length_seconds(),
                shared_terms,
            }
        })
        .collect();

    SuccessResponse::new(related)
}
