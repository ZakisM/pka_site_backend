use serde::Serialize;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
#[schema(bound = "T: utoipa::ToSchema + serde::Serialize + std::marker::Send")]
pub struct SuccessResponse<T>
where
    T: serde::Serialize + std::marker::Send,
{
    pub code: u16,
    pub data: T,
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
