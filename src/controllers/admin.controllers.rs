use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::PgPool;

use crate::middlewares::AdminUser;
use crate::utils::AuthError;

pub struct AdminController;

impl AdminController {
    /// Get admin dashboard
    /// GET /admin/dashboard
    /// Requires: Admin role
    pub async fn get_dashboard(
        admin: AdminUser,
    ) -> Result<impl IntoResponse, AuthError> {
        let response = json!({
            "message": "Welcome to Admin Dashboard",
            "user_id": admin.user_id,
            "email": admin.email,
            "role": "admin",
            "permissions": [
                "manage_users",
                "manage_schools",
                "manage_mentors",
                "manage_students",
                "view_reports",
                "system_settings"
            ]
        });

        Ok((StatusCode::OK, Json(response)))
    }

    /// Get all users (admin only)
    /// GET /admin/users
    /// Requires: Admin role
    pub async fn get_all_users(
        admin: AdminUser,
        State(pool): State<PgPool>,
    ) -> Result<impl IntoResponse, AuthError> {
        let users = sqlx::query_as::<_, (String, String, String)>(
            "SELECT id::text, email, role FROM users ORDER BY created_at DESC"
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        let response = json!({
            "admin_id": admin.user_id,
            "total_users": users.len(),
            "users": users.iter().map(|(id, email, role)| {
                json!({
                    "id": id,
                    "email": email,
                    "role": role
                })
            }).collect::<Vec<_>>()
        });

        Ok((StatusCode::OK, Json(response)))
    }

    /// Get user statistics
    /// GET /admin/statistics
    /// Requires: Admin role
    pub async fn get_statistics(
        admin: AdminUser,
        State(pool): State<PgPool>,
    ) -> Result<impl IntoResponse, AuthError> {
        let total_users = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users"
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        let admin_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE role = 'admin'"
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        let student_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE role = 'student'"
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        let mentor_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE role = 'mentor'"
        )
        .fetch_one(&pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        let response = json!({
            "admin_id": admin.user_id,
            "statistics": {
                "total_users": total_users,
                "admins": admin_count,
                "students": student_count,
                "mentors": mentor_count
            }
        });

        Ok((StatusCode::OK, Json(response)))
    }

    /// Deactivate user (admin only)
    /// POST /admin/users/:user_id/deactivate
    /// Requires: Admin role
    pub async fn deactivate_user(
        admin: AdminUser,
        State(pool): State<PgPool>,
        axum::extract::Path(user_id): axum::extract::Path<String>,
    ) -> Result<impl IntoResponse, AuthError> {
        sqlx::query(
            "UPDATE users SET is_active = false WHERE id = $1"
        )
        .bind(&user_id)
        .execute(&pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        let response = json!({
            "message": "User deactivated successfully",
            "admin_id": admin.user_id,
            "deactivated_user_id": user_id
        });

        Ok((StatusCode::OK, Json(response)))
    }

    /// Activate user (admin only)
    /// POST /admin/users/:user_id/activate
    /// Requires: Admin role
    pub async fn activate_user(
        admin: AdminUser,
        State(pool): State<PgPool>,
        axum::extract::Path(user_id): axum::extract::Path<String>,
    ) -> Result<impl IntoResponse, AuthError> {
        sqlx::query(
            "UPDATE users SET is_active = true WHERE id = $1"
        )
        .bind(&user_id)
        .execute(&pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        let response = json!({
            "message": "User activated successfully",
            "admin_id": admin.user_id,
            "activated_user_id": user_id
        });

        Ok((StatusCode::OK, Json(response)))
    }
}
