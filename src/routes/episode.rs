use warp::filters::BoxedFilter;
use warp::Filter;

use crate::routes::api_path_prefix as main_prefix;
use crate::{handlers, StateFilter};

fn episodes_root() -> BoxedFilter<()> {
    main_prefix().and(warp::path("episodes")).boxed()
}

fn watch_pka_episode_r(
    state: StateFilter,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    episodes_root()
        .and(warp::path::param::<f32>())
        .and(warp::path::end())
        .and(warp::get())
        .and(state)
        .and_then(handlers::episode::watch_pka_episode)
        .boxed()
}

fn find_pka_episode_youtube_link_r(
    state: StateFilter,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    episodes_root()
        .and(warp::path::param::<f32>())
        .and(warp::path("youtube-link"))
        .and(warp::path::end())
        .and(warp::get())
        .and(state)
        .and_then(handlers::episode::find_pka_episode_youtube_link)
        .boxed()
}

fn latest_pka_episode_r(
    state: StateFilter,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    episodes_root()
        .and(warp::path("latest"))
        .and(warp::path::end())
        .and(warp::get())
        .and(state)
        .and_then(handlers::episode::latest_pka_episode)
        .boxed()
}

fn random_pka_episode_r(
    state: StateFilter,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    episodes_root()
        .and(warp::path("random"))
        .and(warp::path::end())
        .and(warp::get())
        .and(state)
        .and_then(handlers::episode::random_pka_episode)
        .boxed()
}

pub fn episode_routes(
    state: StateFilter,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let state_c = || state.clone();

    watch_pka_episode_r(state_c())
        .or(latest_pka_episode_r(state_c()))
        .or(random_pka_episode_r(state_c()))
        .or(find_pka_episode_youtube_link_r(state_c()))
}
