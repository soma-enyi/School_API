use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::models::Claims;
use crate::utils::AuthError;

/// Middleware to enforce admin role
pub async fn require_admin(
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    check_role(request, next, &["admin"]).await
}

/// Middleware to enforce student role
pub async fn require_student(
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    check_role(request, next, &["student"]).await
}

/// Middleware to enforce mentor role
pub async fn require_mentor(
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    check_role(request, next, &["mentor"]).await
}

/// Middleware to enforce multiple roles
pub async fn require_roles(
    request: Request,
    next: Next,
    allowed_roles: &[&str],
) -> Result<Response, AuthError> {
    check_role(request, next, allowed_roles).await
}

/// Helper function to check if user has required role
async fn check_role(
    request: Request,
    next: Next,
    required_roles: &[&str],
) -> Result<Response, AuthError> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AuthError::Unauthorized)?;

    if !required_roles.contains(&claims.role.as_str()) {
        return Err(AuthError::Forbidden);
    }

    Ok(next.run(request).await)
}

/// Response wrapper for role-based errors
pub struct RoleGuardError {
    pub status: StatusCode,
    pub message: String,
}

impl IntoResponse for RoleGuardError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "error": self.message,
            "status": self.status.as_u16(),
        }));

        (self.status, body).into_response()
    }
}
