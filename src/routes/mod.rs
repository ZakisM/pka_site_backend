use warp::filters::BoxedFilter;
use warp::Filter;

pub mod episode;
pub mod events;
pub mod search;

fn path_prefix() -> BoxedFilter<()> {
    warp::path!("v1" / "api" / ..).boxed()
}
