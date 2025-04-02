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
    AddUpdateKeybindingError

}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("->> {:<12} - {self:?}", "ERROR");
        match self {
            Self::LoginFailError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
            }
            Self::DatabaseConnectionError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_SERVER_ERROR").into_response()
            }
            Self::UserNotFoundError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
            }
            Self::UserCreationError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
            }
            Self::InvalidRequestFormatError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
            }
            Self::MigrationExecutionError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
            }
            Self::MigrationKeyError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
            }
            Self::UserUpdateError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
            }
            Self::DocumentNotFoundError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
            }
            Self::PermissionError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "PERMISSION_ERROR").into_response()
            }
            Self::DocumentUpdateError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
            }
            Self::EmailAlreadyExistsError => {
                (StatusCode::CONFLICT, "EMAIL_ALREADY_EXISTS").into_response()
            }
            Self::DatabaseError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR").into_response()
            }
            Self::UserIdUpdateError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "USER_ID_NOT_FOUND").into_response()
            }
            Self::DocumentCreationError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
            }
            Self::PermissionCreationError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
            }
            Self::DocumentDeletionError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
            }
            Self::ProjectNotFoundError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
            }
            Self::DeleteKeybindingError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()   
            }
            Self::AddUpdateKeybindingError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response() 
            }
            Self::ProfilePicError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response() 
            }
            Self::ProfilePicSizeError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response() 
            }
        }
    }
}
