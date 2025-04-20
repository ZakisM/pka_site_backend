use std::sync::Arc;

use warp::Rejection;

use crate::conduit::sqlite::pka_event;
use crate::models::errors::ApiError;
use crate::models::success_response::SuccessResponse;
use crate::Repo;

pub async fn random_pka_event(state: Arc<Repo>) -> Result<impl warp::Reply, Rejection> {
    let random_events = pka_event::random_amount(&state)
        .await
        .map_err(|_| ApiError::new_internal_error("Couldn't find random events."))?;

    Ok(SuccessResponse::new(random_events))
}
