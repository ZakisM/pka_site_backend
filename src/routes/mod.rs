use warp::filters::BoxedFilter;
use warp::Filter;

pub mod episode;
pub mod events;
pub mod search;
pub mod static_files;

fn api_path_prefix() -> BoxedFilter<()> {
    warp::path!("v1" / "api" / ..).boxed()
}
