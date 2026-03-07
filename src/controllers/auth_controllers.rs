use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::{
    AuthResponse, LoginRequest, RefreshTokenRequest, RegisterRequest, UserResponse,
};
use crate::services::{AuthService, EmailService, OtpService};
use crate::utils::{AuthError, JwtConfig};

/// Request body for OTP verification
#[derive(serde::Deserialize, ToSchema)]
#[schema(example = json!({
    "email": "admin@school.com",
    "otp": "123456"
}))]
pub struct OtpVerificationRequest {
    #[schema(example = "admin@school.com", format = "email")]
    pub email: String,
    
    #[schema(example = "123456", min_length = 6, max_length = 6)]
    pub otp: String,
}

/// Request body for resending OTP
#[derive(serde::Deserialize, ToSchema)]
#[schema(example = json!({
    "email": "admin@school.com"
}))]
pub struct ResendOtpRequest {
    #[schema(example = "admin@school.com", format = "email")]
    pub email: String,
}

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

        let user = AuthService::register(&pool, req.clone()).await?;

        // Send welcome email
        let email_config = crate::services::EmailConfig::from_env()?;
        let email_service = EmailService::new(email_config);
        
        let full_name = format!("{} {}", user.first_name, user.last_name);
        if let Err(e) = email_service
            .send_welcome_email(&user.email, &full_name, &user.role)
            .await
        {
            tracing::warn!("Failed to send welcome email: {:?}", e);
        }

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

    /// Login as Admin with OTP
    /// POST /auth/admin/login
    pub async fn login_admin(
        State((pool, _jwt_config)): State<(PgPool, JwtConfig)>,
        Json(req): Json<LoginRequest>,
    ) -> Result<impl IntoResponse, AuthError> {
        let user = AuthService::login(&pool, req).await?;

        // Verify user is admin
        AuthService::validate_role(&user.role, &["admin"])?;

        // Generate and send OTP
        let otp = OtpService::create_otp(&pool, user.id, "login", 10).await?;

        let email_config = crate::services::EmailConfig::from_env()?;
        let email_service = EmailService::new(email_config);
        
        let full_name = format!("{} {}", user.first_name, user.last_name);
        email_service
            .send_otp_email(&user.email, &otp, &full_name)
            .await?;

        let response = json!({
            "message": "OTP sent to your email",
            "user_id": user.id,
            "email": user.email,
            "requires_otp": true
        });

        Ok((StatusCode::OK, Json(response)))
    }

    /// Verify OTP for login
    /// POST /auth/verify-otp
    #[utoipa::path(
        post,
        path = "/auth/verify-otp",
        tag = "Authentication",
        request_body = OtpVerificationRequest,
        responses(
            (status = 200, description = "OTP verified successfully, returns authentication tokens", body = AuthResponse),
            (status = 400, description = "Invalid OTP or expired", body = crate::utils::ErrorResponse),
            (status = 404, description = "User not found", body = crate::utils::ErrorResponse),
        )
    )]
    pub async fn verify_otp_login(
        State((pool, jwt_config)): State<(PgPool, JwtConfig)>,
        Json(req): Json<OtpVerificationRequest>,
    ) -> Result<impl IntoResponse, AuthError> {
        // Find user by email
        let user = sqlx::query_as::<_, crate::models::User>(
            "SELECT id, email, password_hash, first_name, last_name, role, is_active, created_at, updated_at FROM users WHERE email = $1"
        )
        .bind(&req.email)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::InvalidCredentials)?;

        // Verify OTP
        OtpService::verify_otp(&pool, user.id, &req.otp, "login").await?;

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

    /// Resend OTP
    /// POST /auth/resend-otp
    #[utoipa::path(
        post,
        path = "/auth/resend-otp",
        tag = "Authentication",
        request_body = ResendOtpRequest,
        responses(
            (status = 200, description = "OTP resent successfully"),
            (status = 404, description = "User not found", body = crate::utils::ErrorResponse),
            (status = 500, description = "Failed to send email", body = crate::utils::ErrorResponse),
        )
    )]
    pub async fn resend_otp(
        State((pool, _jwt_config)): State<(PgPool, JwtConfig)>,
        Json(req): Json<ResendOtpRequest>,
    ) -> Result<impl IntoResponse, AuthError> {
        let user = sqlx::query_as::<_, crate::models::User>(
            "SELECT id, email, password_hash, first_name, last_name, role, is_active, created_at, updated_at FROM users WHERE email = $1"
        )
        .bind(&req.email)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::UserNotFound)?;

        let otp = OtpService::resend_otp(&pool, user.id, "login").await?;

        let email_config = crate::services::EmailConfig::from_env()?;
        let email_service = EmailService::new(email_config);
        
        let full_name = format!("{} {}", user.first_name, user.last_name);
        email_service
            .send_otp_email(&user.email, &otp, &full_name)
            .await?;

        let response = json!({
            "message": "OTP resent to your email",
            "email": user.email
        });

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

        // Send welcome email
        let email_config = crate::services::EmailConfig::from_env()?;
        let email_service = EmailService::new(email_config);
        
        let full_name = format!("{} {}", user.first_name, user.last_name);
        if let Err(e) = email_service
            .send_welcome_email(&user.email, &full_name, &user.role)
            .await
        {
            tracing::warn!("Failed to send welcome email: {:?}", e);
        }

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

        // Send welcome email
        let email_config = crate::services::EmailConfig::from_env()?;
        let email_service = EmailService::new(email_config);
        
        let full_name = format!("{} {}", user.first_name, user.last_name);
        if let Err(e) = email_service
            .send_welcome_email(&user.email, &full_name, &user.role)
            .await
        {
            tracing::warn!("Failed to send welcome email: {:?}", e);
        }

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
