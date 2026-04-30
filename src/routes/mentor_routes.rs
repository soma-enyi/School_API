use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::Value;
use sqlx::PgPool;

use crate::controllers::mentor;
use crate::controllers::mentor::{CreateAssignmentPayload, GradeAssignmentPayload};
use crate::middlewares::MentorUser;
use crate::utils::AuthError;

// ─── Route Definitions ───

/// Mentor routes — all require mentor role (enforced via MentorUser extractor)
pub fn mentor_routes() -> Router<PgPool> {
    Router::new()
        .route("/mentor/dashboard", get(get_dashboard))
        .route("/mentor/profile", get(get_profile))
        .route("/mentor/students", get(get_students))
        .route(
            "/mentor/students/:student_id/progress",
            get(get_student_progress),
        )
        .route(
            "/mentor/assignments/:assignment_id/grade",
            post(grade_assignment),
        )
        .route("/mentor/assignments/create", post(create_assignment))
        .route(
            "/mentor/messages/student/:student_id",
            post(message_student),
        )
        .route(
            "/mentor/courses/:course_id/assignments",
            get(get_course_assignments),
        )
}

// ─── Handlers ───

/// Get mentor dashboard
#[utoipa::path(
    get,
    path = "/mentor/dashboard",
    tag = "Mentor",
    responses(
        (status = 200, description = "Dashboard data retrieved successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Mentor role required", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_dashboard(
    mentor_user: MentorUser,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, AuthError> {
    let data = mentor::dashboard(&pool, mentor_user.user_id, &mentor_user.email).await?;
    Ok((StatusCode::OK, Json(data)))
}

/// Get mentor profile
#[utoipa::path(
    get,
    path = "/mentor/profile",
    tag = "Mentor",
    responses(
        (status = 200, description = "Profile retrieved successfully", body = crate::models::UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_profile(
    mentor_user: MentorUser,
    axum::extract::State(pool): axum::extract::State<PgPool>,
) -> Result<impl IntoResponse, AuthError> {
    let data = mentor::profile(&pool, mentor_user.user_id).await?;
    Ok((StatusCode::OK, Json(data)))
}

/// Get assigned students
#[utoipa::path(
    get,
    path = "/mentor/students",
    tag = "Mentor",
    responses(
        (status = 200, description = "Students retrieved successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_students(
    mentor_user: MentorUser,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, AuthError> {
    let data = mentor::students(&pool, mentor_user.user_id, &mentor_user.email).await?;
    Ok((StatusCode::OK, Json(data)))
}

/// Get student progress
#[utoipa::path(
    get,
    path = "/mentor/students/{student_id}/progress",
    tag = "Mentor",
    params(("student_id" = String, Path, description = "Student UUID")),
    responses(
        (status = 200, description = "Student progress retrieved successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Student not found", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_student_progress(
    mentor_user: MentorUser,
    State(pool): State<PgPool>,
    Path(student_id): Path<String>,
) -> Result<impl IntoResponse, AuthError> {
    let data = mentor::student_progress(&pool, mentor_user.user_id, &student_id).await?;
    Ok((StatusCode::OK, Json(data)))
}

/// Grade assignment
#[utoipa::path(
    post,
    path = "/mentor/assignments/{assignment_id}/grade",
    tag = "Mentor",
    params(("assignment_id" = String, Path, description = "Assignment UUID")),
    request_body = GradeAssignmentPayload,
    responses(
        (status = 200, description = "Assignment graded successfully"),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Assignment not found", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn grade_assignment(
    mentor_user: MentorUser,
    State(pool): State<PgPool>,
    Path(assignment_id): Path<String>,
    Json(payload): Json<GradeAssignmentPayload>,
) -> Result<impl IntoResponse, AuthError> {
    let data = mentor::grade_assignment(&pool, mentor_user.user_id, &assignment_id, payload).await?;
    Ok((StatusCode::OK, Json(data)))
}

/// Create assignment
#[utoipa::path(
    post,
    path = "/mentor/assignments/create",
    tag = "Mentor",
    request_body = CreateAssignmentPayload,
    responses(
        (status = 201, description = "Assignment created successfully"),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_assignment(
    mentor_user: MentorUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateAssignmentPayload>,
) -> Result<impl IntoResponse, AuthError> {
    let data = mentor::create_assignment(&pool, mentor_user.user_id, payload).await?;
    Ok((StatusCode::CREATED, Json(data)))
}

/// Message student
#[utoipa::path(
    post,
    path = "/mentor/messages/student/{student_id}",
    tag = "Mentor",
    params(("student_id" = String, Path, description = "Student UUID")),
    responses(
        (status = 201, description = "Message sent successfully"),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Student not found", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn message_student(
    mentor_user: MentorUser,
    Path(student_id): Path<String>,
    Json(_payload): Json<Value>,
) -> Result<impl IntoResponse, AuthError> {
    let data = mentor::message_student(mentor_user.user_id, &student_id);
    Ok((StatusCode::CREATED, Json(data)))
}

/// Get course assignments
#[utoipa::path(
    get,
    path = "/mentor/courses/{course_id}/assignments",
    tag = "Mentor",
    params(("course_id" = String, Path, description = "Course UUID")),
    responses(
        (status = 200, description = "Course assignments retrieved successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Course not found", body = ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_course_assignments(
    mentor_user: MentorUser,
    State(pool): State<PgPool>,
    Path(course_id): Path<String>,
) -> Result<impl IntoResponse, AuthError> {
    let data = mentor::course_assignments(&pool, mentor_user.user_id, &course_id).await?;
    Ok((StatusCode::OK, Json(data)))
}
