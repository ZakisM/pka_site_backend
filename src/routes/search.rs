use warp::filters::BoxedFilter;
use warp::Filter;

use crate::models::search::SearchQuery;
use crate::routes::api_path_prefix as main_prefix;
use crate::{handlers, RedisFilter, StateFilter};

fn search_root() -> BoxedFilter<()> {
    main_prefix().and(warp::path("search")).boxed()
}

fn search_pka_episode_r(
    state: StateFilter,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    search_root()
        .and(warp::path("episodes"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::content_length_limit(64))
        .and(warp::body::json::<SearchQuery>())
        .and(state)
        .and_then(handlers::search::search_pka_episode)
        .boxed()
}

fn search_pka_event_r(
    redis: RedisFilter,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    search_root()
        .and(warp::path("events"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::content_length_limit(64))
        .and(warp::body::json::<SearchQuery>())
        .and(redis)
        .and_then(handlers::search::search_pka_event)
        .boxed()
}

pub fn search_routes(
    state: StateFilter,
    redis: RedisFilter,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let state_c = || state.clone();
    let redis_c = || redis.clone();

    search_pka_episode_r(state_c()).or(search_pka_event_r(redis_c()))
}
