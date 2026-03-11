use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::Value;
use sqlx::PgPool;

use crate::controllers::student;
use crate::middlewares::StudentUser;
use crate::utils::AuthError;

// ─── Route Definitions ───

/// Student routes — all require student role (enforced via StudentUser extractor)
pub fn student_routes() -> Router<PgPool> {
    Router::new()
        .route("/student/dashboard", get(get_dashboard))
        .route("/student/profile", get(get_profile))
        .route("/student/courses", get(get_courses))
        .route(
            "/student/assignments/:assignment_id/submit",
            post(submit_assignment),
        )
        .route("/student/grades", get(get_grades))
        .route("/student/messages/mentor", post(message_mentor))
}

// ─── Handlers ───

/// Get student dashboard
#[utoipa::path(
    get,
    path = "/student/dashboard",
    tag = "Student",
    responses(
        (status = 200, description = "Dashboard data retrieved successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Student role required", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_dashboard(
    student_user: StudentUser,
) -> Result<impl IntoResponse, AuthError> {
    let data = student::dashboard(student_user.user_id, &student_user.email);
    Ok((StatusCode::OK, Json(data)))
}

/// Get student profile
#[utoipa::path(
    get,
    path = "/student/profile",
    tag = "Student",
    responses(
        (status = 200, description = "Profile retrieved successfully", body = crate::models::UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_profile(
    student_user: StudentUser,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, AuthError> {
    let data = student::profile(&pool, student_user.user_id).await?;
    Ok((StatusCode::OK, Json(data)))
}

/// Get enrolled courses
#[utoipa::path(
    get,
    path = "/student/courses",
    tag = "Student",
    responses(
        (status = 200, description = "Courses retrieved successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_courses(
    student_user: StudentUser,
) -> Result<impl IntoResponse, AuthError> {
    let data = student::courses(student_user.user_id, &student_user.email);
    Ok((StatusCode::OK, Json(data)))
}

/// Submit assignment
#[utoipa::path(
    post,
    path = "/student/assignments/{assignment_id}/submit",
    tag = "Student",
    params(("assignment_id" = String, Path, description = "Assignment UUID")),
    responses(
        (status = 201, description = "Assignment submitted successfully"),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Assignment not found", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn submit_assignment(
    student_user: StudentUser,
    Path(assignment_id): Path<String>,
    Json(_payload): Json<Value>,
) -> Result<impl IntoResponse, AuthError> {
    let data = student::submit_assignment(student_user.user_id, &assignment_id);
    Ok((StatusCode::CREATED, Json(data)))
}

/// Get grades
#[utoipa::path(
    get,
    path = "/student/grades",
    tag = "Student",
    responses(
        (status = 200, description = "Grades retrieved successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_grades(
    student_user: StudentUser,
) -> Result<impl IntoResponse, AuthError> {
    let data = student::grades(student_user.user_id, &student_user.email);
    Ok((StatusCode::OK, Json(data)))
}

/// Message mentor
#[utoipa::path(
    post,
    path = "/student/messages/mentor",
    tag = "Student",
    responses(
        (status = 201, description = "Message sent successfully"),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn message_mentor(
    student_user: StudentUser,
    Json(_payload): Json<Value>,
) -> Result<impl IntoResponse, AuthError> {
    let data = student::message_mentor(student_user.user_id);
    Ok((StatusCode::CREATED, Json(data)))
}
