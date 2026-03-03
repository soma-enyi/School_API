use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    AuthResponse, LoginRequest, RefreshTokenRequest, RegisterRequest, UserResponse,
};
use crate::services::AuthService;
use crate::utils::{AuthError, JwtConfig};

pub struct AuthController;

/// Register a new Admin user
#[utoipa::path(
        post,
        path = "/auth/admin/register",
        tag = "Authentication",
        request_body = RegisterRequest,
        responses(
            (status = 201, description = "Admin registered successfully", body = AuthResponse),
            (status = 400, description = "Invalid request data", body = crate::utils::ErrorResponse),
            (status = 409, description = "Email already exists", body = crate::utils::ErrorResponse),
            (status = 500, description = "Internal server error", body = crate::utils::ErrorResponse),
        )
    )]
pub async fn register_admin(
        State((pool, jwt_config)): State<(PgPool, JwtConfig)>,
        Json(mut req): Json<RegisterRequest>,
    ) -> Result<impl IntoResponse, AuthError> {
        // Force role to admin
        req.role = "admin".to_string();

        let user = AuthService::register(&pool, req).await?;

        let (access_token, refresh_token) = crate::utils::generate_tokens(
            user.id,
            user.email.clone(),
            user.role.clone(),
            &jwt_config,
        )
        .map_err(|_| AuthError::InternalServerError)?;

        let response = AuthResponse {
            user,
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: jwt_config.access_token_expiry,
        };

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
            (status = 401, description = "Invalid credentials", body = crate::utils::ErrorResponse),
            (status = 403, description = "Account deactivated", body = crate::utils::ErrorResponse),
        )
    )]
pub async fn login_admin(
        State((pool, jwt_config)): State<(PgPool, JwtConfig)>,
        Json(req): Json<LoginRequest>,
    ) -> Result<impl IntoResponse, AuthError> {
        let user = AuthService::login(&pool, req).await?;

        // Verify user is admin
        AuthService::validate_role(&user.role, &["admin"])?;

        let (access_token, refresh_token) = crate::utils::generate_tokens(
            user.id,
            user.email.clone(),
            user.role.clone(),
            &jwt_config,
        )
        .map_err(|_| AuthError::InternalServerError)?;

        let response = AuthResponse {
            user: UserResponse::from(user),
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: jwt_config.access_token_expiry,
        };

    Ok((StatusCode::OK, Json(response)))
}

/// Register a new Student user
#[utoipa::path(
        post,
        path = "/auth/student/register",
        tag = "Authentication",
        request_body = RegisterRequest,
        responses(
            (status = 201, description = "Student registered successfully", body = AuthResponse),
            (status = 400, description = "Invalid request data", body = crate::utils::ErrorResponse),
            (status = 409, description = "Email already exists", body = crate::utils::ErrorResponse),
            (status = 500, description = "Internal server error", body = crate::utils::ErrorResponse),
        )
    )]
pub async fn register_student(
        State((pool, jwt_config)): State<(PgPool, JwtConfig)>,
        Json(mut req): Json<RegisterRequest>,
    ) -> Result<impl IntoResponse, AuthError> {
        // Force role to student
        req.role = "student".to_string();

        let user = AuthService::register(&pool, req).await?;

        let (access_token, refresh_token) = crate::utils::generate_tokens(
            user.id,
            user.email.clone(),
            user.role.clone(),
            &jwt_config,
        )
        .map_err(|_| AuthError::InternalServerError)?;

        let response = AuthResponse {
            user,
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: jwt_config.access_token_expiry,
        };

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
            (status = 401, description = "Invalid credentials", body = crate::utils::ErrorResponse),
            (status = 403, description = "Account deactivated", body = crate::utils::ErrorResponse),
        )
    )]
pub async fn login_student(
        State((pool, jwt_config)): State<(PgPool, JwtConfig)>,
        Json(req): Json<LoginRequest>,
    ) -> Result<impl IntoResponse, AuthError> {
        let user = AuthService::login(&pool, req).await?;

        // Verify user is student
        AuthService::validate_role(&user.role, &["student"])?;

        let (access_token, refresh_token) = crate::utils::generate_tokens(
            user.id,
            user.email.clone(),
            user.role.clone(),
            &jwt_config,
        )
        .map_err(|_| AuthError::InternalServerError)?;

        let response = AuthResponse {
            user: UserResponse::from(user),
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: jwt_config.access_token_expiry,
        };

    Ok((StatusCode::OK, Json(response)))
}

/// Register a new Mentor user
#[utoipa::path(
        post,
        path = "/auth/mentor/register",
        tag = "Authentication",
        request_body = RegisterRequest,
        responses(
            (status = 201, description = "Mentor registered successfully", body = AuthResponse),
            (status = 400, description = "Invalid request data", body = crate::utils::ErrorResponse),
            (status = 409, description = "Email already exists", body = crate::utils::ErrorResponse),
            (status = 500, description = "Internal server error", body = crate::utils::ErrorResponse),
        )
    )]
pub async fn register_mentor(
        State((pool, jwt_config)): State<(PgPool, JwtConfig)>,
        Json(mut req): Json<RegisterRequest>,
    ) -> Result<impl IntoResponse, AuthError> {
        // Force role to mentor
        req.role = "mentor".to_string();

        let user = AuthService::register(&pool, req).await?;

        let (access_token, refresh_token) = crate::utils::generate_tokens(
            user.id,
            user.email.clone(),
            user.role.clone(),
            &jwt_config,
        )
        .map_err(|_| AuthError::InternalServerError)?;

        let response = AuthResponse {
            user,
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: jwt_config.access_token_expiry,
        };

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
            (status = 401, description = "Invalid credentials", body = crate::utils::ErrorResponse),
            (status = 403, description = "Account deactivated", body = crate::utils::ErrorResponse),
        )
    )]
pub async fn login_mentor(
        State((pool, jwt_config)): State<(PgPool, JwtConfig)>,
        Json(req): Json<LoginRequest>,
    ) -> Result<impl IntoResponse, AuthError> {
        let user = AuthService::login(&pool, req).await?;

        // Verify user is mentor
        AuthService::validate_role(&user.role, &["mentor"])?;

        let (access_token, refresh_token) = crate::utils::generate_tokens(
            user.id,
            user.email.clone(),
            user.role.clone(),
            &jwt_config,
        )
        .map_err(|_| AuthError::InternalServerError)?;

        let response = AuthResponse {
            user: UserResponse::from(user),
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: jwt_config.access_token_expiry,
        };

    Ok((StatusCode::OK, Json(response)))
}

/// Refresh access token
#[utoipa::path(
        post,
        path = "/auth/refresh",
        tag = "Authentication",
        request_body = RefreshTokenRequest,
        responses(
            (status = 200, description = "Token refreshed successfully"),
            (status = 401, description = "Invalid or expired refresh token", body = crate::utils::ErrorResponse),
        )
    )]
pub async fn refresh_token(
        State((pool, jwt_config)): State<(PgPool, JwtConfig)>,
        Json(req): Json<RefreshTokenRequest>,
    ) -> Result<impl IntoResponse, AuthError> {
        // Verify refresh token
        let claims = crate::utils::verify_token(&req.refresh_token, &jwt_config)
            .map_err(|_| AuthError::InvalidToken)?;

        // Ensure it's a refresh token
        if claims.token_type != "refresh" {
            return Err(AuthError::InvalidToken);
        }

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AuthError::InvalidToken)?;

        let (access_token, expires_in) =
            AuthService::refresh_access_token(&pool, user_id, &jwt_config).await?;

        let response = json!({
            "access_token": access_token,
            "token_type": "Bearer",
            "expires_in": expires_in,
        });

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
        security(
            ("bearer_auth" = [])
        )
    )]
pub async fn logout() -> impl IntoResponse {
        let response = json!({
            "message": "Logged out successfully. Please discard the tokens on client side."
        });

    (StatusCode::OK, Json(response))
}

/// Get current user profile
#[utoipa::path(
        get,
        path = "/auth/me",
        tag = "Authentication",
        responses(
            (status = 200, description = "User profile retrieved successfully", body = UserResponse),
            (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
pub async fn get_current_user(
        State((pool, jwt_config)): State<(PgPool, JwtConfig)>,
        headers: axum::http::HeaderMap,
    ) -> Result<impl IntoResponse, AuthError> {
        // Extract token from Authorization header
        let auth_header = headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or(AuthError::Unauthorized)?;

        let token = crate::utils::extract_token_from_header(auth_header)
            .ok_or(AuthError::InvalidToken)?;

        // Verify token
        let claims = crate::utils::verify_token(&token, &jwt_config)
            .map_err(|_| AuthError::InvalidToken)?;

        // Ensure it's an access token
        if claims.token_type != "access" {
            return Err(AuthError::InvalidToken);
        }

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AuthError::InvalidToken)?;

        let user = AuthService::get_user_by_id(&pool, user_id).await?;

    Ok((StatusCode::OK, Json(UserResponse::from(user))))
}

/// Verify token validity
#[utoipa::path(
        post,
        path = "/auth/verify",
        tag = "Authentication",
        responses(
            (status = 200, description = "Token is valid"),
            (status = 401, description = "Invalid or expired token", body = crate::utils::ErrorResponse),
        ),
        security(
            ("bearer_auth" = [])
        )
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

        let claims = crate::utils::verify_token(&token, &jwt_config)
            .map_err(|_| AuthError::InvalidToken)?;

        let response = json!({
            "valid": true,
            "user_id": claims.sub,
            "email": claims.email,
            "role": claims.role,
            "token_type": claims.token_type,
        });

    Ok((StatusCode::OK, Json(response)))
}
