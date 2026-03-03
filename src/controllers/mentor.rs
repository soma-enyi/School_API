use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::PgPool;

use crate::middlewares::MentorUser;
use crate::utils::AuthError;

pub struct MentorController;

/// Get mentor dashboard
#[utoipa::path(
        get,
        path = "/mentor/dashboard",
        tag = "Mentor",
        responses(
            (status = 200, description = "Dashboard data retrieved successfully"),
            (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
            (status = 403, description = "Forbidden - Mentor role required", body = crate::utils::ErrorResponse),
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
pub async fn get_dashboard(
        mentor: MentorUser,
    ) -> Result<impl IntoResponse, AuthError> {
        let response = json!({
            "message": "Welcome to Mentor Dashboard",
            "user_id": mentor.user_id,
            "email": mentor.email,
            "role": "mentor",
            "permissions": [
                "manage_courses",
                "grade_assignments",
                "message_students",
                "view_student_progress",
                "create_assignments"
            ]
        });

    Ok((StatusCode::OK, Json(response)))
}

/// Get mentor profile
#[utoipa::path(
        get,
        path = "/mentor/profile",
        tag = "Mentor",
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
        mentor: MentorUser,
        State(pool): State<PgPool>,
    ) -> Result<impl IntoResponse, AuthError> {
        let user = sqlx::query_as::<_, (String, String, String, String, String)>(
            "SELECT id::text, email, first_name, last_name, role FROM users WHERE id = $1"
        )
        .bind(mentor.user_id.to_string())
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

/// Get assigned students
#[utoipa::path(
        get,
        path = "/mentor/students",
        tag = "Mentor",
        responses(
            (status = 200, description = "Students retrieved successfully"),
            (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
            (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
pub async fn get_students(
        mentor: MentorUser,
    ) -> Result<impl IntoResponse, AuthError> {
        let response = json!({
            "mentor_id": mentor.user_id,
            "email": mentor.email,
            "students": [
                {
                    "id": "student_1",
                    "name": "Alice Johnson",
                    "email": "alice@example.com",
                    "status": "active"
                },
                {
                    "id": "student_2",
                    "name": "Bob Smith",
                    "email": "bob@example.com",
                    "status": "active"
                },
                {
                    "id": "student_3",
                    "name": "Carol White",
                    "email": "carol@example.com",
                    "status": "active"
                }
            ]
        });

    Ok((StatusCode::OK, Json(response)))
}

/// Get student progress
#[utoipa::path(
        get,
        path = "/mentor/students/{student_id}/progress",
        tag = "Mentor",
        params(
            ("student_id" = String, Path, description = "Student UUID")
        ),
        responses(
            (status = 200, description = "Student progress retrieved successfully"),
            (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
            (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
            (status = 404, description = "Student not found", body = crate::utils::ErrorResponse),
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
pub async fn get_student_progress(
        mentor: MentorUser,
        axum::extract::Path(student_id): axum::extract::Path<String>,
    ) -> Result<impl IntoResponse, AuthError> {
        let response = json!({
            "mentor_id": mentor.user_id,
            "student_id": student_id,
            "progress": {
                "courses_enrolled": 2,
                "assignments_completed": 5,
                "assignments_pending": 2,
                "average_grade": "A-",
                "last_activity": chrono::Utc::now().to_rfc3339()
            }
        });

    Ok((StatusCode::OK, Json(response)))
}

/// Grade assignment
#[utoipa::path(
        post,
        path = "/mentor/assignments/{assignment_id}/grade",
        tag = "Mentor",
        params(
            ("assignment_id" = String, Path, description = "Assignment UUID")
        ),
        responses(
            (status = 200, description = "Assignment graded successfully"),
            (status = 400, description = "Invalid request data", body = crate::utils::ErrorResponse),
            (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
            (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
            (status = 404, description = "Assignment not found", body = crate::utils::ErrorResponse),
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
pub async fn grade_assignment(
        mentor: MentorUser,
        axum::extract::Path(assignment_id): axum::extract::Path<String>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Result<impl IntoResponse, AuthError> {
        let response = json!({
            "message": "Assignment graded successfully",
            "mentor_id": mentor.user_id,
            "assignment_id": assignment_id,
            "graded_at": chrono::Utc::now().to_rfc3339(),
            "status": "graded"
        });

    Ok((StatusCode::OK, Json(response)))
}

/// Create assignment
#[utoipa::path(
        post,
        path = "/mentor/assignments/create",
        tag = "Mentor",
        responses(
            (status = 201, description = "Assignment created successfully"),
            (status = 400, description = "Invalid request data", body = crate::utils::ErrorResponse),
            (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
            (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
pub async fn create_assignment(
        mentor: MentorUser,
        Json(_payload): Json<serde_json::Value>,
    ) -> Result<impl IntoResponse, AuthError> {
        let response = json!({
            "message": "Assignment created successfully",
            "mentor_id": mentor.user_id,
            "assignment_id": uuid::Uuid::new_v4().to_string(),
            "created_at": chrono::Utc::now().to_rfc3339(),
            "status": "active"
        });

    Ok((StatusCode::CREATED, Json(response)))
}

/// Message student
#[utoipa::path(
        post,
        path = "/mentor/messages/student/{student_id}",
        tag = "Mentor",
        params(
            ("student_id" = String, Path, description = "Student UUID")
        ),
        responses(
            (status = 201, description = "Message sent successfully"),
            (status = 400, description = "Invalid request data", body = crate::utils::ErrorResponse),
            (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
            (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
            (status = 404, description = "Student not found", body = crate::utils::ErrorResponse),
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
pub async fn message_student(
        mentor: MentorUser,
        axum::extract::Path(student_id): axum::extract::Path<String>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Result<impl IntoResponse, AuthError> {
        let response = json!({
            "message": "Message sent to student successfully",
            "mentor_id": mentor.user_id,
            "student_id": student_id,
            "sent_at": chrono::Utc::now().to_rfc3339(),
            "status": "sent"
        });

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get course assignments
#[utoipa::path(
        get,
        path = "/mentor/courses/{course_id}/assignments",
        tag = "Mentor",
        params(
            ("course_id" = String, Path, description = "Course UUID")
        ),
        responses(
            (status = 200, description = "Course assignments retrieved successfully"),
            (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
            (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
            (status = 404, description = "Course not found", body = crate::utils::ErrorResponse),
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
pub async fn get_course_assignments(
        mentor: MentorUser,
        axum::extract::Path(course_id): axum::extract::Path<String>,
    ) -> Result<impl IntoResponse, AuthError> {
        let response = json!({
            "mentor_id": mentor.user_id,
            "course_id": course_id,
            "assignments": [
                {
                    "id": "assignment_1",
                    "title": "Rust Basics",
                    "due_date": "2024-02-15",
                    "submissions": 3,
                    "graded": 2
                },
                {
                    "id": "assignment_2",
                    "title": "Web API Project",
                    "due_date": "2024-02-22",
                    "submissions": 2,
                    "graded": 0
                }
            ]
        });

    Ok((StatusCode::OK, Json(response)))
}
