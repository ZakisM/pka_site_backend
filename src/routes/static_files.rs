use warp::Filter;

use crate::{handlers, StateFilter};

fn robots_txt_r() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("robots.txt")
        .and(warp::get())
        .and_then(handlers::static_files::robots_txt)
        .boxed()
}

fn sitemap_xml_r(
    state: StateFilter,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("sitemap.xml")
        .and(warp::get())
        .and(state)
        .and_then(handlers::static_files::sitemap_xml)
        .boxed()
}

pub fn static_files_routes(
    state: StateFilter,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let state_c = || state.clone();

    robots_txt_r().or(sitemap_xml_r(state_c()))
}
