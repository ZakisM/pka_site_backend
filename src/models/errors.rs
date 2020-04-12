use std::fmt;
use std::time::SystemTimeError;

use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use warp::http::StatusCode;
use warp::reply::Response;

#[derive(Clone, Debug)]
pub struct ApiError {
    message: String,
    code: StatusCode,
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

impl From<std::time::SystemTimeError> for ApiError {
    fn from(std: SystemTimeError) -> Self {
        error!("{}", std.to_string());
        ApiError::new("Server Time Error", StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<std::io::Error> for ApiError {
    fn from(std: std::io::Error) -> Self {
        error!("{}", std.to_string());
        ApiError::new("IO Error", StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<diesel::result::Error> for ApiError {
    fn from(de_err: diesel::result::Error) -> Self {
        let err_string = de_err.to_string();
        let description = de_err.to_string();
        error!("{}", err_string);

        match description.as_str() {
            "Record not found" => ApiError::new("Not found in database", StatusCode::NOT_FOUND),
            _ => ApiError::new(err_string, StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

impl warp::Reply for ApiError {
    fn into_response(self) -> Response {
        let json = warp::reply::json(&self);
        warp::reply::with_status(json, self.code).into_response()
    }
}

impl From<serde_json::error::Error> for ApiError {
    fn from(se: serde_json::error::Error) -> Self {
        ApiError::new(se.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<regex::Error> for ApiError {
    fn from(re: regex::Error) -> Self {
        error!("{}", re.to_string());
        ApiError::new(re.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(req: reqwest::Error) -> Self {
        ApiError::new(req.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<feed_rs::parser::ParseFeedError> for ApiError {
    fn from(_: feed_rs::parser::ParseFeedError) -> Self {
        ApiError::new("Couldn't read RSS feed.", StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<std::string::FromUtf8Error> for ApiError {
    fn from(utfe: std::string::FromUtf8Error) -> Self {
        ApiError::new(utfe.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<redis::RedisError> for ApiError {
    fn from(rede: redis::RedisError) -> Self {
        ApiError::new(rede.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
    }
}
