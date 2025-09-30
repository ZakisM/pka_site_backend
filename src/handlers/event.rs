use axum::extract::State;

use crate::app_state::AppState;
use crate::conduit::sqlite::pka_event;
use crate::models::errors::ApiError;
use crate::models::search::PkaEventSearchResult;
use crate::models::success_response::SuccessResponse;

pub async fn random_pka_event(
    State(state): State<AppState>,
) -> Result<SuccessResponse<Option<PkaEventSearchResult>>, ApiError> {
    let random_events = pka_event::random_amount(state.db.as_ref())
        .await
        .map_err(|_| ApiError::new_internal_error("Couldn't find random events."))?;

    Ok(SuccessResponse::new(random_events))
}
