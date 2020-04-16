use warp::filters::BoxedFilter;
use warp::Filter;

use crate::routes::path_prefix as main_prefix;
use crate::{handlers, StateFilter};

fn path_prefix() -> BoxedFilter<()> {
    main_prefix().and(warp::path!("pka_episode" / ..)).boxed()
}

fn watch_pka_episode_r(
    state: StateFilter,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    path_prefix()
        .and(warp::path!("watch" / f32))
        .and(warp::get())
        .and(state)
        .and_then(handlers::episode::watch_pka_episode)
        .boxed()
}

fn find_pka_episode_youtube_link_r(
    state: StateFilter,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    path_prefix()
        .and(warp::path!("youtube_link" / f32))
        .and(warp::get())
        .and(state)
        .and_then(handlers::episode::find_pka_episode_youtube_link)
        .boxed()
}

fn latest_pka_episode_r(
    state: StateFilter,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    path_prefix()
        .and(warp::path!("watch" / "latest"))
        .and(warp::get())
        .and(state)
        .and_then(handlers::episode::latest_pka_episode)
        .boxed()
}

fn random_pka_episode_r(
    state: StateFilter,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    path_prefix()
        .and(warp::path!("watch" / "random"))
        .and(warp::get())
        .and(state)
        .and_then(handlers::episode::random_pka_episode)
        .boxed()
}

pub fn episode_routes(
    state: StateFilter,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let state_c = || state.clone();

    watch_pka_episode_r(state_c())
        .or(latest_pka_episode_r(state_c()))
        .or(random_pka_episode_r(state_c()))
        .or(find_pka_episode_youtube_link_r(state_c()))
}
