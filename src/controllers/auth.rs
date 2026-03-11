use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{AuthResponse, LoginRequest, RegisterRequest, UserResponse};
use crate::services::AuthService;
use crate::utils::{AuthError, JwtConfig};

/// Register a new user with the given role and return an auth response with tokens.
pub async fn register_user(
    pool: &PgPool,
    mut req: RegisterRequest,
    role: &str,
    jwt_config: &JwtConfig,
) -> Result<AuthResponse, AuthError> {
    req.role = role.to_string();

    let user = AuthService::register(pool, req).await?;

    let (access_token, refresh_token) = crate::utils::generate_tokens(
        user.id,
        user.email.clone(),
        user.role.clone(),
        jwt_config,
    )
    .map_err(|_| AuthError::InternalServerError)?;

    Ok(AuthResponse {
        user,
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: jwt_config.access_token_expiry,
    })
}

/// Authenticate a user, validate their role, and return an auth response with tokens.
pub async fn login_user(
    pool: &PgPool,
    req: LoginRequest,
    required_roles: &[&str],
    jwt_config: &JwtConfig,
) -> Result<AuthResponse, AuthError> {
    let user = AuthService::login(pool, req).await?;

    AuthService::validate_role(&user.role, required_roles)?;

    let (access_token, refresh_token) = crate::utils::generate_tokens(
        user.id,
        user.email.clone(),
        user.role.clone(),
        jwt_config,
    )
    .map_err(|_| AuthError::InternalServerError)?;

    Ok(AuthResponse {
        user: UserResponse::from(user),
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: jwt_config.access_token_expiry,
    })
}

/// Verify a refresh token and generate a new access token.
pub async fn refresh_access_token(
    pool: &PgPool,
    refresh_token_str: &str,
    jwt_config: &JwtConfig,
) -> Result<Value, AuthError> {
    let claims = crate::utils::verify_token(refresh_token_str, jwt_config)
        .map_err(|_| AuthError::InvalidToken)?;

    if claims.token_type != "refresh" {
        return Err(AuthError::InvalidToken);
    }

    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken)?;

    let (access_token, expires_in) =
        AuthService::refresh_access_token(pool, user_id, jwt_config).await?;

    Ok(json!({
        "access_token": access_token,
        "token_type": "Bearer",
        "expires_in": expires_in,
    }))
}

/// Get user profile from an access token.
pub async fn get_profile(
    pool: &PgPool,
    token: &str,
    jwt_config: &JwtConfig,
) -> Result<UserResponse, AuthError> {
    let claims = crate::utils::verify_token(token, jwt_config)
        .map_err(|_| AuthError::InvalidToken)?;

    if claims.token_type != "access" {
        return Err(AuthError::InvalidToken);
    }

    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken)?;

    let user = AuthService::get_user_by_id(pool, user_id).await?;

    Ok(UserResponse::from(user))
}

/// Verify a token and return its claims.
pub fn verify_token_claims(
    token: &str,
    jwt_config: &JwtConfig,
) -> Result<Value, AuthError> {
    let claims = crate::utils::verify_token(token, jwt_config)
        .map_err(|_| AuthError::InvalidToken)?;

    Ok(json!({
        "valid": true,
        "user_id": claims.sub,
        "email": claims.email,
        "role": claims.role,
        "token_type": claims.token_type,
    }))
}
