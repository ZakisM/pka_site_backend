use serde::Serialize;
use warp::http::StatusCode;
use warp::reply::Response;

#[derive(Serialize)]
pub struct SuccessResponse<T>
where
    T: serde::Serialize + std::marker::Send,
{
    code: u16,
    data: T,
}

impl<T> SuccessResponse<T>
where
    T: serde::Serialize + std::marker::Send,
{
    pub fn new(data: T) -> Self {
        Self {
            data,
            code: StatusCode::OK.as_u16(),
        }
    }
}

impl<T> warp::Reply for SuccessResponse<T>
where
    T: serde::Serialize + std::marker::Send,
{
    fn into_response(self) -> Response {
        let json = warp::reply::json(&self);
        warp::reply::with_status(json, StatusCode::OK).into_response()
    }
}
