use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use sqlx::PgPool;

use crate::controllers::auth;
use crate::models::{LoginRequest, RefreshTokenRequest, RegisterRequest};
use crate::utils::{AuthError, JwtConfig};

// ─── Route Definitions ───

pub fn auth_routes(pool: PgPool, jwt_config: JwtConfig) -> Router {
    Router::new()
        // Admin authentication
        .route("/admin/register", post(register_admin))
        .route("/admin/login", post(login_admin))
        // Student authentication
        .route("/student/register", post(register_student))
        .route("/student/login", post(login_student))
        // Mentor authentication
        .route("/mentor/register", post(register_mentor))
        .route("/mentor/login", post(login_mentor))
        // Common
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout))
        .route("/me", get(get_current_user))
        .route("/verify", post(verify_token_endpoint))
        .with_state((pool, jwt_config))
}

// ─── Admin Auth Handlers ───

/// Register a new Admin user
#[utoipa::path(
    post,
    path = "/auth/admin/register",
    tag = "Authentication",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Admin registered successfully", body = AuthResponse),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 409, description = "Email already exists", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn register_admin(
    State((pool, jwt_config)): State<(PgPool, JwtConfig)>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let response = auth::register_user(&pool, req, "admin", &jwt_config).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// Login as Admin
#[utoipa::path(
    post,
    path = "/auth/admin/login",
    tag = "Authentication",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 403, description = "Account deactivated", body = ErrorResponse),
    )
)]
pub async fn login_admin(
    State((pool, jwt_config)): State<(PgPool, JwtConfig)>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let response = auth::login_user(&pool, req, &["admin"], &jwt_config).await?;
    Ok((StatusCode::OK, Json(response)))
}

// ─── Student Auth Handlers ───

/// Register a new Student user
#[utoipa::path(
    post,
    path = "/auth/student/register",
    tag = "Authentication",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Student registered successfully", body = AuthResponse),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 409, description = "Email already exists", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn register_student(
    State((pool, jwt_config)): State<(PgPool, JwtConfig)>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let response = auth::register_user(&pool, req, "student", &jwt_config).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// Login as Student
#[utoipa::path(
    post,
    path = "/auth/student/login",
    tag = "Authentication",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 403, description = "Account deactivated", body = ErrorResponse),
    )
)]
pub async fn login_student(
    State((pool, jwt_config)): State<(PgPool, JwtConfig)>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let response = auth::login_user(&pool, req, &["student"], &jwt_config).await?;
    Ok((StatusCode::OK, Json(response)))
}

// ─── Mentor Auth Handlers ───

/// Register a new Mentor user
#[utoipa::path(
    post,
    path = "/auth/mentor/register",
    tag = "Authentication",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Mentor registered successfully", body = AuthResponse),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 409, description = "Email already exists", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    )
)]
pub async fn register_mentor(
    State((pool, jwt_config)): State<(PgPool, JwtConfig)>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let response = auth::register_user(&pool, req, "mentor", &jwt_config).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// Login as Mentor
#[utoipa::path(
    post,
    path = "/auth/mentor/login",
    tag = "Authentication",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 403, description = "Account deactivated", body = ErrorResponse),
    )
)]
pub async fn login_mentor(
    State((pool, jwt_config)): State<(PgPool, JwtConfig)>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let response = auth::login_user(&pool, req, &["mentor"], &jwt_config).await?;
    Ok((StatusCode::OK, Json(response)))
}

// ─── Common Auth Handlers ───

/// Refresh access token
#[utoipa::path(
    post,
    path = "/auth/refresh",
    tag = "Authentication",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token refreshed successfully"),
        (status = 401, description = "Invalid or expired refresh token", body = ErrorResponse),
    )
)]
pub async fn refresh_token(
    State((pool, jwt_config)): State<(PgPool, JwtConfig)>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let response =
        auth::refresh_access_token(&pool, &req.refresh_token, &jwt_config).await?;
    Ok((StatusCode::OK, Json(response)))
}

/// Logout user
#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "Authentication",
    responses(
        (status = 200, description = "Logged out successfully"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn logout() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "message": "Logged out successfully. Please discard the tokens on client side."
        })),
    )
}

/// Get current user profile
#[utoipa::path(
    get,
    path = "/auth/me",
    tag = "Authentication",
    responses(
        (status = 200, description = "User profile retrieved successfully", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_current_user(
    State((pool, jwt_config)): State<(PgPool, JwtConfig)>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, AuthError> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(AuthError::Unauthorized)?;

    let token = crate::utils::extract_token_from_header(auth_header)
        .ok_or(AuthError::InvalidToken)?;

    let user = auth::get_profile(&pool, &token, &jwt_config).await?;
    Ok((StatusCode::OK, Json(user)))
}

/// Verify token validity
#[utoipa::path(
    post,
    path = "/auth/verify",
    tag = "Authentication",
    responses(
        (status = 200, description = "Token is valid"),
        (status = 401, description = "Invalid or expired token", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn verify_token_endpoint(
    State((_pool, jwt_config)): State<(PgPool, JwtConfig)>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, AuthError> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(AuthError::Unauthorized)?;

    let token = crate::utils::extract_token_from_header(auth_header)
        .ok_or(AuthError::InvalidToken)?;

    let response = auth::verify_token_claims(&token, &jwt_config)?;
    Ok((StatusCode::OK, Json(response)))
}
