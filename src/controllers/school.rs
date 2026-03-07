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

pub struct SchoolController;

/// Get all schools (admin only)
#[utoipa::path(
    get,
    path = "/admin/schools",
    tag = "School",
    responses(
        (status = 200, description = "List of all schools"),
        (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
        (status = 403, description = "Forbidden - Admin role required", body = crate::utils::ErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_all_schools(
    admin: AdminUser,
    State(_pool): State<PgPool>,
) -> Result<impl IntoResponse, AuthError> {
    let response = json!({
        "admin_id": admin.user_id,
        "schools": [
            {
                "id": "school_1",
                "name": "Central High School",
                "location": "New York",
                "students": 500,
                "mentors": 25
            },
            {
                "id": "school_2",
                "name": "Tech Academy",
                "location": "San Francisco",
                "students": 300,
                "mentors": 15
            }
        ]
    });

    Ok((StatusCode::OK, Json(response)))
}

/// Get school details
#[utoipa::path(
    get,
    path = "/admin/schools/{school_id}",
    tag = "School",
    params(
        ("school_id" = String, Path, description = "School unique identifier")
    ),
    responses(
        (status = 200, description = "School details retrieved successfully"),
        (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
        (status = 403, description = "Forbidden - Admin role required", body = crate::utils::ErrorResponse),
        (status = 404, description = "School not found", body = crate::utils::ErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_school_details(
    admin: AdminUser,
    axum::extract::Path(school_id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, AuthError> {
    let response = json!({
        "admin_id": admin.user_id,
        "school": {
            "id": school_id,
            "name": "Central High School",
            "location": "New York",
            "principal": "Dr. John Smith",
            "email": "principal@school.edu",
            "phone": "+1-555-0123",
            "students": 500,
            "mentors": 25,
            "courses": 15,
            "established": "2010"
        }
    });

    Ok((StatusCode::OK, Json(response)))
}

/// Create new school
#[utoipa::path(
    post,
    path = "/admin/schools/create",
    tag = "School",
    responses(
        (status = 201, description = "School created successfully"),
        (status = 400, description = "Invalid request", body = crate::utils::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
        (status = 403, description = "Forbidden - Admin role required", body = crate::utils::ErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_school(
    admin: AdminUser,
    Json(_payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AuthError> {
    let response = json!({
        "message": "School created successfully",
        "admin_id": admin.user_id,
        "school_id": uuid::Uuid::new_v4().to_string(),
        "created_at": chrono::Utc::now().to_rfc3339()
    });

    Ok((StatusCode::CREATED, Json(response)))
}

/// Update school
#[utoipa::path(
    put,
    path = "/admin/schools/{school_id}",
    tag = "School",
    params(
        ("school_id" = String, Path, description = "School unique identifier")
    ),
    responses(
        (status = 200, description = "School updated successfully"),
        (status = 400, description = "Invalid request", body = crate::utils::ErrorResponse),
        (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
        (status = 403, description = "Forbidden - Admin role required", body = crate::utils::ErrorResponse),
        (status = 404, description = "School not found", body = crate::utils::ErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_school(
    admin: AdminUser,
    axum::extract::Path(school_id): axum::extract::Path<String>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AuthError> {
    let response = json!({
        "message": "School updated successfully",
        "admin_id": admin.user_id,
        "school_id": school_id,
        "updated_at": chrono::Utc::now().to_rfc3339()
    });

    Ok((StatusCode::OK, Json(response)))
}

/// Delete school
#[utoipa::path(
    delete,
    path = "/admin/schools/{school_id}",
    tag = "School",
    params(
        ("school_id" = String, Path, description = "School unique identifier")
    ),
    responses(
        (status = 200, description = "School deleted successfully"),
        (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
        (status = 403, description = "Forbidden - Admin role required", body = crate::utils::ErrorResponse),
        (status = 404, description = "School not found", body = crate::utils::ErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_school(
    admin: AdminUser,
    axum::extract::Path(school_id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, AuthError> {
    let response = json!({
        "message": "School deleted successfully",
        "admin_id": admin.user_id,
        "school_id": school_id,
        "deleted_at": chrono::Utc::now().to_rfc3339()
    });

    Ok((StatusCode::OK, Json(response)))
}

/// Get school statistics
#[utoipa::path(
    get,
    path = "/admin/schools/{school_id}/statistics",
    tag = "School",
    params(
        ("school_id" = String, Path, description = "School unique identifier")
    ),
    responses(
        (status = 200, description = "School statistics retrieved successfully"),
        (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
        (status = 403, description = "Forbidden - Admin role required", body = crate::utils::ErrorResponse),
        (status = 404, description = "School not found", body = crate::utils::ErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_school_statistics(
    admin: AdminUser,
    axum::extract::Path(school_id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, AuthError> {
    let response = json!({
        "admin_id": admin.user_id,
        "school_id": school_id,
        "statistics": {
            "total_students": 500,
            "total_mentors": 25,
            "total_courses": 15,
            "active_courses": 12,
            "average_class_size": 20,
            "student_retention_rate": "95%"
        }
    });

    Ok((StatusCode::OK, Json(response)))
}
