use axum::extract::State;
use axum::response::IntoResponse;
use rand::seq::IndexedRandom;

use crate::app_state::AppState;
use crate::handlers::NO_STORE;
use crate::models::errors::{ApiError, ErrorResponseBody};
use crate::models::pka_event::RandomPkaEvent;
use crate::models::success_response::SuccessResponse;

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
pub async fn random_pka_event(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let catalog = state.catalog.get();
    let random_event = catalog
        .random_pool
        .choose(&mut rand::rng())
        .and_then(|&position| catalog.events.get(position as usize))
        .map(RandomPkaEvent::from)
        .ok_or_else(|| ApiError::new_internal_error("Couldn't find random events."))?;

    Ok((NO_STORE, SuccessResponse::new(random_event)))
}
