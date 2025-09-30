use serde::Serialize;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

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

impl<T> IntoResponse for SuccessResponse<T>
where
    T: serde::Serialize + std::marker::Send,
{
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}
