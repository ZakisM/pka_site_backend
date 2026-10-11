use rand::seq::IndexedRandom;

use crate::models::errors::{ApiError, ErrorResponseBody};
use crate::models::pka_event::RandomPkaEvent;
use crate::models::success_response::SuccessResponse;
use crate::search::SEARCH_INDEX;

#[utoipa::path(
    get,
    path = "/api/v1/events/random",
    responses(
        (
            status = 200,
            description = "Random event",
            body = SuccessResponse<RandomPkaEvent>
        ),
        (status = 500, description = "Internal server error", body = ErrorResponseBody)
    ),
    tag = "Events"
)]
pub async fn random_pka_event() -> Result<SuccessResponse<RandomPkaEvent>, ApiError> {
    // A pick from the eligible moments the search index already knows,
    // Instead of reading every moment from SQLite on each call.
    let index = SEARCH_INDEX.read().await;
    let random_event = index
        .random_pool
        .choose(&mut rand::rng())
        .and_then(|&position| index.events.get(position as usize))
        .map(RandomPkaEvent::from)
        .ok_or_else(|| ApiError::new_internal_error("Couldn't find random events."))?;

    Ok(SuccessResponse::new(random_event))
}
