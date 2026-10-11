use axum::Json;
use axum::extract::rejection::{PathRejection, QueryRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;
use tracing::error;
use utoipa::ToSchema;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("{message}")]
    WithStatus { code: StatusCode, message: String },
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
        }
    }

    fn message(&self) -> String {
        match self {
            ApiError::WithStatus { message, .. } => message.clone(),
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

impl From<QueryRejection> for ApiError {
    fn from(rejection: QueryRejection) -> Self {
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
