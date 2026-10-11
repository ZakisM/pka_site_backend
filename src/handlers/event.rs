use axum::extract::State;

use crate::app_state::AppState;
use crate::conduit::sqlite::pka_event;
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
) -> Result<SuccessResponse<RandomPkaEvent>, ApiError> {
    let random_event = pka_event::random_amount(state.db.as_ref())
        .await?
        .ok_or_else(|| ApiError::new_internal_error("Couldn't find random events."))?;

    Ok(SuccessResponse::new(random_event))
}
