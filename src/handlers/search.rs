use axum::extract::State;
use axum::http::StatusCode;

use crate::app_state::AppState;
use crate::catalog::search::search;

use crate::extractors::AppQuery;
use crate::models::errors::{ApiError, ErrorResponseBody};
use crate::models::search::{SearchPage, SearchParams};
use crate::models::success_response::SuccessResponse;

// Generous for any real search, small enough that a paste can't be expensive.
const MAX_QUERY_CHARS: usize = 200;

#[utoipa::path(
    get,
    path = "/api/v1/search",
    params(SearchParams),
    responses(
        (status = 200, description = "One page of results grouped by episode", body = SuccessResponse<SearchPage>),
        (status = 400, description = "Invalid search request", body = ErrorResponseBody)
    ),
    tag = "Search"
)]
pub async fn search_pka(
    AppQuery(params): AppQuery<SearchParams>,
    State(state): State<AppState>,
) -> Result<SuccessResponse<SearchPage>, ApiError> {
    if params.q.chars().count() > MAX_QUERY_CHARS {
        return Err(ApiError::new(
            format!("Search queries are limited to {MAX_QUERY_CHARS} characters."),
            StatusCode::BAD_REQUEST,
        ));
    }

    Ok(SuccessResponse::new(search(
        &state.catalog.get(),
        &params.q,
        params.sort,
        params.page,
    )))
}
