use std::convert::Infallible;
use std::sync::Arc;

use warp::Reply;

use crate::models::search::SearchQuery;
use crate::models::success_response::SuccessResponse;
use crate::search::pka_search::{search_episode, search_events};
use crate::Repo;

pub async fn search_pka_episode(
    sq: SearchQuery,
    state: Arc<Repo>,
) -> Result<impl warp::Reply, Infallible> {
    match search_episode(&state, &sq.query).await {
        Ok(res) => Ok(SuccessResponse::new(res).into_response()),
        Err(e) => {
            // Should return Err once improves in Warp;
            Ok(e.into_response())
        }
    }
}

pub async fn search_pka_event(
    sq: SearchQuery,
    state: Arc<Repo>,
) -> Result<impl warp::Reply, Infallible> {
    match search_events(&state, &sq.query).await {
        Ok(res) => Ok(SuccessResponse::new(res).into_response()),
        Err(e) => {
            // Should return Err once improves in Warp;
            Ok(e.into_response())
        }
    }
}
