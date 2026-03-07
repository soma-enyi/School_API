use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{LoginRequest, RegisterRequest, User, UserResponse};
use crate::utils::{AuthError, JwtConfig};

pub struct AuthService;

impl AuthService {
    /// Register a new user with the specified role
    pub async fn register(
        pool: &PgPool,
        req: RegisterRequest,
    ) -> Result<UserResponse, AuthError> {
        // Validate role
        let valid_roles = ["admin", "student", "mentor"];
        if !valid_roles.contains(&req.role.as_str()) {
            return Err(AuthError::InvalidRole);
        }

        // Check if user already exists
        let existing_user = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, first_name, last_name, role, is_active, created_at, updated_at FROM users WHERE email = $1"
        )
        .bind(&req.email)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        if existing_user.is_some() {
            return Err(AuthError::UserAlreadyExists);
        }

        // Hash password
        let password_hash = hash(&req.password, DEFAULT_COST)
            .map_err(|_| AuthError::InternalServerError)?;

        let user_id = Uuid::new_v4();
        let now = Utc::now();

        // Insert user into database
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (id, email, password_hash, first_name, last_name, role, is_active, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING id, email, password_hash, first_name, last_name, role, is_active, created_at, updated_at"
        )
        .bind(user_id)
        .bind(&req.email)
        .bind(&password_hash)
        .bind(&req.first_name)
        .bind(&req.last_name)
        .bind(&req.role)
        .bind(true)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(UserResponse::from(user))
    }

    /// Login user and return user data
    pub async fn login(
        pool: &PgPool,
        req: LoginRequest,
    ) -> Result<User, AuthError> {
        // Find user by email
        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, first_name, last_name, role, is_active, created_at, updated_at FROM users WHERE email = $1"
        )
        .bind(&req.email)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::InvalidCredentials)?;

        // Check if user is active
        if !user.is_active {
            return Err(AuthError::Unauthorized);
        }

        // Verify password
        verify(&req.password, &user.password_hash)
            .map_err(|_| AuthError::InvalidCredentials)?;

        Ok(user)
    }

    /// Get user by ID
    pub async fn get_user_by_id(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<User, AuthError> {
        sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, first_name, last_name, role, is_active, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::UserNotFound)
    }

    /// Get user by email
    pub async fn get_user_by_email(
        pool: &PgPool,
        email: &str,
    ) -> Result<User, AuthError> {
        sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, first_name, last_name, role, is_active, created_at, updated_at FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::UserNotFound)
    }

    /// Verify refresh token and generate new access token
    pub async fn refresh_access_token(
        pool: &PgPool,
        user_id: Uuid,
        config: &JwtConfig,
    ) -> Result<(String, i64), AuthError> {
        let user = Self::get_user_by_id(pool, user_id).await?;

        let (access_token, _) = crate::utils::generate_tokens(
            user.id,
            user.email,
            user.role,
            config,
        )
        .map_err(|_| AuthError::InternalServerError)?;

        Ok((access_token, config.access_token_expiry))
    }

    /// Validate user role for authorization
    pub fn validate_role(user_role: &str, required_roles: &[&str]) -> Result<(), AuthError> {
        if required_roles.contains(&user_role) {
            Ok(())
        } else {
            Err(AuthError::Forbidden)
        }
    }
}
