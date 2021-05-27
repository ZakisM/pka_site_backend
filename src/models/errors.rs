use std::fmt;

use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use warp::http::StatusCode;
use warp::reply::Response;

use crate::convert_error;

#[derive(Clone, Debug)]
pub struct ApiError {
    pub message: String,
    pub code: StatusCode,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} - {})", self.code.as_str(), self.message)
    }
}

impl Serialize for ApiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ApiError", 2)?;
        state.serialize_field("message", &self.message)?;
        state.serialize_field("code", &self.code.as_u16())?;
        state.end()
    }
}

impl ApiError {
    pub fn new<S: AsRef<str>>(message: S, code: StatusCode) -> Self {
        Self {
            message: message.as_ref().to_string(),
            code,
        }
    }

    pub fn new_internal_error<S: AsRef<str>>(message: S) -> Self {
        Self {
            message: message.as_ref().to_string(),
            code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl warp::reject::Reject for ApiError {}

impl warp::Reply for ApiError {
    fn into_response(self) -> Response {
        let json = warp::reply::json(&self);
        warp::reply::with_status(json, self.code).into_response()
    }
}

impl From<diesel::result::Error> for ApiError {
    fn from(de_err: diesel::result::Error) -> Self {
        let err_string = de_err.to_string();

        error!("{}", err_string);

        match err_string.as_str() {
            "Record not found" | "NotFound" => {
                ApiError::new("Data could not be found.", StatusCode::NOT_FOUND)
            }
            _ => ApiError::new(err_string, StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

convert_error!(quick_xml::DeError);
convert_error!(chrono::ParseError);
convert_error!(serde_json::error::Error);
convert_error!(regex::Error);
convert_error!(reqwest::Error);
convert_error!(std::string::FromUtf8Error);
convert_error!(redis::RedisError);

#[macro_export]
macro_rules! convert_error {
    ($err_type:ty) => {
        impl From<$err_type> for ApiError {
            fn from(err: $err_type) -> Self {
                let err_str = err.to_string();

                error!("{}", &err_str);

                ApiError::new(err_str, StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    };

    ($err_type:ty, $custom_message:expr) => {
        impl From<$err_type> for ApiError {
            fn from(err: $err_type) -> Self {
                let err_str = err.to_string();

                error!("{}", &err_str);

                ApiError::new($custom_message, StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    };
}
