use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;
use tracing::error;
use utoipa::ToSchema;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("{message}")]
    WithStatus { code: StatusCode, message: String },
    #[error(transparent)]
    QuickXmlDe(#[from] quick_xml::DeError),
    #[error(transparent)]
    QuickXmlSe(#[from] quick_xml::SeError),
    #[error(transparent)]
    Chrono(#[from] chrono::ParseError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl ApiError {
    pub fn new(message: impl Into<String>, code: StatusCode) -> Self {
        Self::WithStatus {
            code,
            message: message.into(),
        }
    }

    pub fn new_internal_error(message: impl Into<String>) -> Self {
        Self::new(message, StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::WithStatus { code, .. } => *code,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn message(&self) -> String {
        match self {
            ApiError::WithStatus { message, .. } => message.clone(),
            _ => self.to_string(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponseBody {
    pub message: String,
    pub code: u16,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();

        if status.is_server_error() {
            error!("{self}");
        }

        let body = ErrorResponseBody {
            message: self.message(),
            code: status.as_u16(),
        };

        (status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        error!("{err}");
        match err {
            sqlx::Error::RowNotFound => {
                ApiError::new("Data could not be found.", StatusCode::NOT_FOUND)
            }
            _ => ApiError::new(err.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        ApiError::new(rejection.body_text(), rejection.status())
    }
}

impl From<PathRejection> for ApiError {
    fn from(rejection: PathRejection) -> Self {
        use axum::extract::path::ErrorKind;

        match rejection {
            PathRejection::FailedToDeserializePathParams(inner) => {
                let kind = inner.into_kind();
                let status = match kind {
                    ErrorKind::UnsupportedType { .. } => StatusCode::INTERNAL_SERVER_ERROR,
                    _ => StatusCode::BAD_REQUEST,
                };
                ApiError::new(kind.to_string(), status)
            }
            PathRejection::MissingPathParams(inner) => {
                ApiError::new(inner.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            }
            other => ApiError::new(other.to_string(), StatusCode::BAD_REQUEST),
        }
    }
}
