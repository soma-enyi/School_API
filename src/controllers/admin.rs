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

/// Get admin dashboard
#[utoipa::path(
        get,
        path = "/admin/dashboard",
        tag = "Admin",
        responses(
            (status = 200, description = "Dashboard data retrieved successfully"),
            (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
            (status = 403, description = "Forbidden - Admin role required", body = crate::utils::ErrorResponse),
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
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

/// Get all users
#[utoipa::path(
        get,
        path = "/admin/users",
        tag = "Admin",
        responses(
            (status = 200, description = "Users retrieved successfully"),
            (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
            (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
            (status = 500, description = "Internal server error", body = crate::utils::ErrorResponse),
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
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
#[utoipa::path(
        get,
        path = "/admin/statistics",
        tag = "Admin",
        responses(
            (status = 200, description = "Statistics retrieved successfully"),
            (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
            (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
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

/// Deactivate user
#[utoipa::path(
        post,
        path = "/admin/users/{user_id}/deactivate",
        tag = "Admin",
        params(
            ("user_id" = String, Path, description = "User UUID to deactivate")
        ),
        responses(
            (status = 200, description = "User deactivated successfully"),
            (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
            (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
            (status = 404, description = "User not found", body = crate::utils::ErrorResponse),
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
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

/// Activate user
#[utoipa::path(
        post,
        path = "/admin/users/{user_id}/activate",
        tag = "Admin",
        params(
            ("user_id" = String, Path, description = "User UUID to activate")
        ),
        responses(
            (status = 200, description = "User activated successfully"),
            (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
            (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
            (status = 404, description = "User not found", body = crate::utils::ErrorResponse),
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
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
