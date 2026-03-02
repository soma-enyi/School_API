use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use uuid::Uuid;

use crate::models::Claims;
use crate::utils::AuthError;

/// Extractor for authenticated user claims
#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
    pub role: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or(AuthError::Unauthorized)?;

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(AuthenticatedUser {
            user_id,
            email: claims.email,
            role: claims.role,
        })
    }
}

/// Extractor for admin users only
#[derive(Clone, Debug)]
pub struct AdminUser {
    pub user_id: Uuid,
    pub email: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or(AuthError::Unauthorized)?;

        if claims.role != "admin" {
            return Err(AuthError::Forbidden);
        }

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(AdminUser {
            user_id,
            email: claims.email,
        })
    }
}

/// Extractor for student users only
#[derive(Clone, Debug)]
pub struct StudentUser {
    pub user_id: Uuid,
    pub email: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for StudentUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or(AuthError::Unauthorized)?;

        if claims.role != "student" {
            return Err(AuthError::Forbidden);
        }

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(StudentUser {
            user_id,
            email: claims.email,
        })
    }
}

/// Extractor for mentor users only
#[derive(Clone, Debug)]
pub struct MentorUser {
    pub user_id: Uuid,
    pub email: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for MentorUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or(AuthError::Unauthorized)?;

        if claims.role != "mentor" {
            return Err(AuthError::Forbidden);
        }

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(MentorUser {
            user_id,
            email: claims.email,
        })
    }
}
