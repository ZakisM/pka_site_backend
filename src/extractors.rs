use axum::extract::FromRequestParts;

use crate::models::errors::ApiError;

#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ApiError))]
pub struct AppPath<T>(pub T);

#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(ApiError))]
pub struct AppQuery<T>(pub T);
