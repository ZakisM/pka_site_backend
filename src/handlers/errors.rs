use std::convert::Infallible;

use warp::filters::body::BodyDeserializeError;
use warp::http::StatusCode;
use warp::reject::PayloadTooLarge;
use warp::Reply;

use crate::models::errors::ApiError;

pub async fn handle_rejection(err: warp::Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND".to_owned();
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED".to_owned();
    } else if err.find::<BodyDeserializeError>().is_some() {
        error!("Could not read JSON data: {:?}", err);
        code = StatusCode::BAD_REQUEST;
        message = "Could not read JSON data".to_owned();
    } else if err.find::<PayloadTooLarge>().is_some() {
        error!("Payload was too large: {:?}", err);
        code = StatusCode::BAD_REQUEST;
        message = "Payload was too large".to_owned();
    } else {
        error!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "INTERNAL_SERVER_ERROR".to_owned();
    }

    Ok(ApiError::new(message, code).into_response())
}
