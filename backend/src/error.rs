pub type Result<T> = core::result::Result<T, Error>; // Export Type

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    // Database Errors
    MigrationExecutionError,
    MigrationKeyError,
    DatabaseConnectionError,

    // User Errors
    UserUpdateError { user_id: i32 },
    UserNotFoundError { user_id: i32 },
    UserCreationError,
    LoginFailError,
    UserIdUpdateError,
    ProfilePicError,
    ProfilePicSizeError,
    PasswordValidationError,

    // Document Errors
    DocumentNotFoundError { document_id: i32 },
    DocumentUpdateError { document_id: i32 },
    DocumentCreationError,
    DocumentDeletionError { document_id: i32 },

    // General Errors
    InvalidRequestFormatError,

    // Document Permission Errors
    PermissionError,
    PermissionCreationError,

    // Signup Errors
    EmailAlreadyExistsError,
    DatabaseError,

    // Project Errors
    ProjectNotFoundError { project_id: i32 },

    // Keybinding Errors
    DeleteKeybindingError { command_id: i32 },
    AddUpdateKeybindingError { command_id: i32 },

    // AI Errors
    EmbeddingError,
    APIKeyError,
    LlmQueryError,
    InsufficientAiCredits,
    FailedApplyChanges,
    
    // Preference Errors
    PreferenceNotFoundError { preference_id: i32 },
}

#[derive(Debug, Clone, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
    LOGIN_FAIL,
    NO_AUTH,
    EMAIL_ALREADY_EXISTS,
    PASSWORD_VALIDATION_ERROR,
    INVALID_PARAMS,
    RESOURCE_NOT_FOUND,
    INSUFFICIENT_AI_CREDITS,
    SERVICE_ERROR,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("->> {:<12} - {self:?}", "INTO_RESPONSE");

        // Create the placeholder for axum response
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into the response
        response.extensions_mut().insert(self);

        response // Return the response with the error extension
    }
}

impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        #[allow(unreachable_patterns)]
        match self {
            // Auth / Login Errors
            Self::LoginFailError => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::LOGIN_FAIL),
            Self::UserNotFoundError { .. } | Self::PermissionError => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),
            Self::EmailAlreadyExistsError => (StatusCode::CONFLICT, ClientError::EMAIL_ALREADY_EXISTS),
            Self::PasswordValidationError => (StatusCode::BAD_REQUEST, ClientError::PASSWORD_VALIDATION_ERROR),

            // Request / Model Errors
            Self::InvalidRequestFormatError => (StatusCode::BAD_REQUEST, ClientError::INVALID_PARAMS),
            Self::ProfilePicSizeError => (StatusCode::BAD_REQUEST, ClientError::INVALID_PARAMS), // Treat size error as bad params
            
            // AI Specific Errors
            Self::InsufficientAiCredits => (StatusCode::PAYMENT_REQUIRED, ClientError::INSUFFICIENT_AI_CREDITS),
            Self::APIKeyError => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR), // Could be config issue
            Self::EmbeddingError => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR),
            Self::LlmQueryError => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR),

            // Apply Suggestion Errors
            Self::FailedApplyChanges { .. } => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR),

            // Resource Errors (Could argue some are Forbidden/No_Auth if based on user context)
            Self::DocumentNotFoundError { .. } => (StatusCode::NOT_FOUND, ClientError::RESOURCE_NOT_FOUND),
            Self::ProjectNotFoundError { .. } => (StatusCode::NOT_FOUND, ClientError::RESOURCE_NOT_FOUND),
            Self::DocumentCreationError => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR),
            Self::DocumentUpdateError { .. } => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR),
            Self::DocumentDeletionError { .. } => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR),
            Self::PermissionCreationError => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR),
            Self::UserCreationError => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR),
            Self::UserUpdateError { .. } => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR),
            Self::UserIdUpdateError => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR), // Internal state issue
            Self::ProfilePicError => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR),
            Self::DeleteKeybindingError { .. } => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR),
            Self::AddUpdateKeybindingError { .. } => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR),
            
            // Database / Migration Errors (Internal Server Errors)
            Self::DatabaseError | 
            Self::DatabaseConnectionError | 
            Self::MigrationExecutionError | 
            Self::MigrationKeyError => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR),

            // Preference Errors
            Self::PreferenceNotFoundError { .. } => (StatusCode::NOT_FOUND, ClientError::RESOURCE_NOT_FOUND),

            // Fallback for any other unmapped error
            _ => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR),
        }
    }
}