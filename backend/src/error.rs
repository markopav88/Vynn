pub type Result<T> = core::result::Result<T, Error>; // Export Type

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;

#[derive(Debug)]
pub enum Error {
    // Database Errors
    MigrationExecutionError,
    MigrationKeyError,
    DatabaseConnectionError,

    // User Errors
    UserUpdateError,
    UserNotFoundError,
    UserCreationError,
    LoginFailError,
    UserIdUpdateError,
    ProfilePicError,
    ProfilePicSizeError,
    PasswordValidationError,

    // Document Errors
    DocumentNotFoundError,
    DocumentUpdateError,
    DocumentCreationError,
    DocumentDeletionError,

    // General Errors
    InvalidRequestFormatError,

    // Document Permission Errors
    PermissionError,
    PermissionCreationError,

    // Signup Errors
    EmailAlreadyExistsError,
    DatabaseError,

    // Project Errors
    ProjectNotFoundError,

    // Keybinding Errors
    DeleteKeybindingError,
    AddUpdateKeybindingError,

    // AI Errors
    EmbeddingError,
    APIKeyError,
    LlmQueryError,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("->> {:<12} - {self:?}", "ERROR");
        let (status, error_message) = match self {
            Self::LoginFailError => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR"),
            Self::DatabaseConnectionError => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_SERVER_ERROR"),
            Self::UserNotFoundError => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR"),
            Self::UserCreationError => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR"),
            Self::InvalidRequestFormatError => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR"),
            Self::MigrationExecutionError => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR"),
            Self::MigrationKeyError => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR"),
            Self::UserUpdateError => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR"),
            Self::DocumentNotFoundError => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR"),
            Self::PermissionError => (StatusCode::INTERNAL_SERVER_ERROR, "PERMISSION_ERROR"),
            Self::DocumentUpdateError => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR"),
            Self::EmailAlreadyExistsError => (StatusCode::CONFLICT, "EMAIL_ALREADY_EXISTS"),
            Self::DatabaseError => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
            Self::UserIdUpdateError => (StatusCode::INTERNAL_SERVER_ERROR, "USER_ID_NOT_FOUND"),
            Self::DocumentCreationError => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR"),
            Self::PermissionCreationError => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR"),
            Self::DocumentDeletionError => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR"),
            Self::ProjectNotFoundError => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR"),
            Self::DeleteKeybindingError => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR"),
            Self::AddUpdateKeybindingError => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR"),
            Self::ProfilePicError => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR"),
            Self::ProfilePicSizeError => (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR"),
            Self::PasswordValidationError => (StatusCode::BAD_REQUEST, "PASSWORD_VALIDATION_ERROR"),
            Self::EmbeddingError => (StatusCode::BAD_REQUEST, "EMBEDDING_ERROR"),
            Self::APIKeyError => (StatusCode::BAD_REQUEST, "API_KEY_ERROR"),
            Self::LlmQueryError => (StatusCode::INTERNAL_SERVER_ERROR, "LLM_QUERY_ERROR"),
        };

        (status, error_message).into_response()
    }
}
