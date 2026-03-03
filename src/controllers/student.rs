use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::PgPool;

use crate::middlewares::StudentUser;
use crate::utils::AuthError;

pub struct StudentController;

/// Get student dashboard
#[utoipa::path(
        get,
        path = "/student/dashboard",
        tag = "Student",
        responses(
            (status = 200, description = "Dashboard data retrieved successfully"),
            (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
            (status = 403, description = "Forbidden - Student role required", body = crate::utils::ErrorResponse),
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
pub async fn get_dashboard(
        student: StudentUser,
    ) -> Result<impl IntoResponse, AuthError> {
        let response = json!({
            "message": "Welcome to Student Dashboard",
            "user_id": student.user_id,
            "email": student.email,
            "role": "student",
            "permissions": [
                "view_courses",
                "submit_assignments",
                "view_grades",
                "message_mentor",
                "view_schedule"
            ]
        });

    Ok((StatusCode::OK, Json(response)))
}

/// Get student profile
#[utoipa::path(
        get,
        path = "/student/profile",
        tag = "Student",
        responses(
            (status = 200, description = "Profile retrieved successfully", body = crate::models::UserResponse),
            (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
            (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
            (status = 404, description = "User not found", body = crate::utils::ErrorResponse),
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
pub async fn get_profile(
        student: StudentUser,
        State(pool): State<PgPool>,
    ) -> Result<impl IntoResponse, AuthError> {
        let user = sqlx::query_as::<_, (String, String, String, String, String)>(
            "SELECT id::text, email, first_name, last_name, role FROM users WHERE id = $1"
        )
        .bind(student.user_id.to_string())
        .fetch_optional(&pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::UserNotFound)?;

        let response = json!({
            "id": user.0,
            "email": user.1,
            "first_name": user.2,
            "last_name": user.3,
            "role": user.4
        });

    Ok((StatusCode::OK, Json(response)))
}

/// Get enrolled courses
#[utoipa::path(
        get,
        path = "/student/courses",
        tag = "Student",
        responses(
            (status = 200, description = "Courses retrieved successfully"),
            (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
            (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
pub async fn get_courses(
        student: StudentUser,
    ) -> Result<impl IntoResponse, AuthError> {
        let response = json!({
            "student_id": student.user_id,
            "email": student.email,
            "courses": [
                {
                    "id": "course_1",
                    "name": "Introduction to Rust",
                    "mentor": "John Doe",
                    "status": "active"
                },
                {
                    "id": "course_2",
                    "name": "Web Development with Axum",
                    "mentor": "Jane Smith",
                    "status": "active"
                }
            ]
        });

    Ok((StatusCode::OK, Json(response)))
}

/// Submit assignment
#[utoipa::path(
        post,
        path = "/student/assignments/{assignment_id}/submit",
        tag = "Student",
        params(
            ("assignment_id" = String, Path, description = "Assignment UUID")
        ),
        responses(
            (status = 201, description = "Assignment submitted successfully"),
            (status = 400, description = "Invalid request data", body = crate::utils::ErrorResponse),
            (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
            (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
            (status = 404, description = "Assignment not found", body = crate::utils::ErrorResponse),
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
pub async fn submit_assignment(
        student: StudentUser,
        axum::extract::Path(assignment_id): axum::extract::Path<String>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Result<impl IntoResponse, AuthError> {
        let response = json!({
            "message": "Assignment submitted successfully",
            "student_id": student.user_id,
            "assignment_id": assignment_id,
            "submitted_at": chrono::Utc::now().to_rfc3339(),
            "status": "pending_review"
        });

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get grades
#[utoipa::path(
        get,
        path = "/student/grades",
        tag = "Student",
        responses(
            (status = 200, description = "Grades retrieved successfully"),
            (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
            (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
pub async fn get_grades(
        student: StudentUser,
    ) -> Result<impl IntoResponse, AuthError> {
        let response = json!({
            "student_id": student.user_id,
            "email": student.email,
            "grades": [
                {
                    "course": "Introduction to Rust",
                    "assignment": "Assignment 1",
                    "grade": "A",
                    "score": 95
                },
                {
                    "course": "Web Development with Axum",
                    "assignment": "Project 1",
                    "grade": "B+",
                    "score": 87
                }
            ]
        });

    Ok((StatusCode::OK, Json(response)))
}

/// Message mentor
#[utoipa::path(
        post,
        path = "/student/messages/mentor",
        tag = "Student",
        responses(
            (status = 201, description = "Message sent successfully"),
            (status = 400, description = "Invalid request data", body = crate::utils::ErrorResponse),
            (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
            (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
pub async fn message_mentor(
        student: StudentUser,
        Json(_payload): Json<serde_json::Value>,
    ) -> Result<impl IntoResponse, AuthError> {
        let response = json!({
            "message": "Message sent to mentor successfully",
            "student_id": student.user_id,
            "sent_at": chrono::Utc::now().to_rfc3339(),
            "status": "sent"
        });

    Ok((StatusCode::CREATED, Json(response)))
}
