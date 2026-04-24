use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

/// Standardized error response structure for all API errors
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "error": "Unauthorized",
    "message": "Invalid or expired token",
    "timestamp": "2024-01-15T10:30:00Z"
}))]
pub struct ErrorResponse {
    #[schema(example = "Unauthorized")]
    pub error: String,
    
    #[schema(example = "Invalid or expired token")]
    pub message: String,
    
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub timestamp: String,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Invalid role")]
    InvalidRole,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Internal server error")]
    InternalServerError,

    #[error("Sign-in window has closed")]
    SignInClosed,

    #[error("Sign-in window not yet open")]
    SignInNotOpen,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_type, error_message) = match self {
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Unauthorized", "Invalid email or password"),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, "NotFound", "User not found"),
            AuthError::UserAlreadyExists => (StatusCode::CONFLICT, "Conflict", "User already exists"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Unauthorized", "Invalid token"),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "Unauthorized", "Token expired"),
            AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized", "Unauthorized"),
            AuthError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden", "Forbidden"),
            AuthError::InvalidRole => (StatusCode::BAD_REQUEST, "BadRequest", "Invalid role"),
            AuthError::DatabaseError(ref msg) => (StatusCode::INTERNAL_SERVER_ERROR, "InternalServerError", msg.as_str()),
            AuthError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "InternalServerError", "Internal server error"),
            AuthError::SignInClosed => (StatusCode::FORBIDDEN, "SignInClosed", "Sign-in window has closed (cutoff is 10:15 AM)"),
            AuthError::SignInNotOpen => (StatusCode::FORBIDDEN, "SignInNotOpen", "Sign-in window is not yet open (opens at 8:00 AM)"),
        };

        let error_response = ErrorResponse::new(error_type, error_message);
        (status, Json(error_response)).into_response()
    }
}
