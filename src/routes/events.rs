use warp::filters::BoxedFilter;
use warp::Filter;

use crate::models::search::SearchQuery;
use crate::routes::api_path_prefix as main_prefix;
use crate::{handlers, RedisFilter, StateFilter};

fn path_prefix() -> BoxedFilter<()> {
    main_prefix().and(warp::path!("events" / ..)).boxed()
}

fn random_pka_event_r(
    state: StateFilter,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    path_prefix()
        .and(warp::path("random"))
        .and(warp::get())
        .and(state)
        .and_then(handlers::event::random_pka_event)
        .boxed()
}

// Copied from src/routes/search.rs and modified
fn search_pka_event_r(
    redis: RedisFilter,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    path_prefix() // Uses the events.rs path_prefix
        .and(warp::path("search")) // New path segment
        .and(warp::post())
        .and(warp::body::content_length_limit(64))
        .and(warp::body::json::<SearchQuery>())
        .and(redis)
        .and_then(crate::handlers::event::search_pka_event) // Updated handler
        .boxed()
}

pub fn event_routes(
    state: StateFilter,
    redis: RedisFilter,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let state_c = || state.clone();
    let redis_c = || redis.clone();

    random_pka_event_r(state_c()).or(search_pka_event_r(redis_c()))
}
