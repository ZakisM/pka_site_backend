use axum::extract::{FromRequest, FromRequestParts};
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

use crate::models::errors::ApiError;

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct AppJson<T>(pub T);

impl<T: Serialize> IntoResponse for AppJson<T> {
    fn into_response(self) -> axum::response::Response {
        Json(self.0).into_response()
    }
}

#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ApiError))]
pub struct AppPath<T>(pub T);
