use warp::filters::BoxedFilter;
use warp::Filter;

use crate::routes::api_path_prefix as main_prefix;
use crate::{handlers, StateFilter};

fn path_prefix() -> BoxedFilter<()> {
    main_prefix().and(warp::path!("events" / ..)).boxed()
}

fn random_pka_events_r(
    state: StateFilter,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    path_prefix()
        .and(warp::path("random"))
        .and(warp::get())
        .and(state)
        .and_then(handlers::event::random_pka_events)
        .boxed()
}

pub fn event_routes(
    state: StateFilter,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let state_c = || state.clone();

    random_pka_events_r(state_c())
}
