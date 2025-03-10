pub type Result<T> = core::result::Result<T, Error>; // Export Type

use axum::response::IntoResponse;
use axum::http::StatusCode;
use axum::response::Response;

#[derive(Debug)]
pub enum Error {
    LoginFail,
    DatabaseConnectionError,
    UserNotFound,
    UserCreationError,
    InvalidRequestFormat,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("->> ERROR: {:?} - {self:?}", "INTO_RESPONSE");
        match self {
            Self::LoginFail => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response(),
            Self::DatabaseConnectionError => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_SERVER_ERROR").into_response(),
            Self::UserNotFound => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response(),
            Self::UserCreationError => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response(),
            Self::InvalidRequestFormat => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response(),
        }
    }
}
