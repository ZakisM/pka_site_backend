use warp::http::Uri;
use warp::Filter;

fn front_end_watch_r() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("watch")
        .and(warp::get())
        .map(|| warp::redirect(Uri::from_static("/")))
}

fn front_end_episodes_r() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::path("episodes")
        .and(warp::get())
        .map(|| warp::redirect(Uri::from_static("/")))
}

fn front_end_events_r() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::path("events")
        .and(warp::get())
        .map(|| warp::redirect(Uri::from_static("/")))
}

pub fn front_end_routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    front_end_watch_r()
        .or(front_end_episodes_r())
        .or(front_end_events_r())
}
