use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use maud::{html, DOCTYPE};

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    NotFound,
    BadRequest(String),
    Unauthorized,
    Forbidden,
    Internal(Box<dyn std::error::Error + Send + Sync>),
}

impl<E: Into<Box<dyn std::error::Error + Send + Sync>>> From<E> for AppError {
    fn from(e: E) -> Self {
        AppError::Internal(e.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found.".to_string()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Please log in.".to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden.".to_string()),
            AppError::Internal(e) => {
                tracing::error!("internal error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error.".to_string(),
                )
            }
        };
        let body = html! {
            (DOCTYPE)
            html {
                head { meta charset="utf-8"; title { (status.as_u16()) } }
                body { p { (message) } }
            }
        };
        (status, body).into_response()
    }
}
