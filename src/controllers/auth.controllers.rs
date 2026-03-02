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

impl AuthController {
    /// Register a new Admin user
    /// POST /auth/admin/register
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
    /// POST /auth/admin/login
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
    /// POST /auth/student/register
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
    /// POST /auth/student/login
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
    /// POST /auth/mentor/register
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
    /// POST /auth/mentor/login
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

    /// Refresh access token for any authenticated user
    /// POST /auth/refresh
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

    /// Logout user (client-side token invalidation)
    /// POST /auth/logout
    pub async fn logout() -> impl IntoResponse {
        let response = json!({
            "message": "Logged out successfully. Please discard the tokens on client side."
        });

        (StatusCode::OK, Json(response))
    }

    /// Get current user profile (requires valid access token)
    /// GET /auth/me
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
    /// POST /auth/verify
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
}
