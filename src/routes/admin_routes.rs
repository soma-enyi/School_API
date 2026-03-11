use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::controllers::admin;
use crate::middlewares::AdminUser;
use crate::models::course::{CreateCourseRequest, UpdateCourseRequest};
use crate::state::AppState;
use crate::utils::AuthError;

// ─── Request DTOs ───

#[derive(Debug, Deserialize)]
pub struct UpdateMentorRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApplicationQuery {
    pub status: Option<String>,
}

// ─── Route Definitions ───

/// Admin routes — all require admin role (enforced via AdminUser extractor)
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        // Dashboard
        .route("/dashboard", get(admin_dashboard))
        // Application review
        .route("/applications", get(list_applications))
        .route("/applications/:id", get(get_application))
        .route("/applications/:id/accept", put(accept_application))
        .route("/applications/:id/waitlist", put(waitlist_application))
        .route("/applications/:id/enroll", put(enroll_application))
        .route("/applications/:id/reject", put(reject_application))
        // Student management
        .route("/students", get(list_students))
        .route("/students/:id", get(get_student))
        .route("/students/:id/waitlist", put(waitlist_student))
        .route("/students/:id/accept", put(accept_student))
        .route("/students/:id/reject", put(reject_student))
        // Mentor management
        .route("/mentors", get(list_mentors))
        .route(
            "/mentors/:id",
            get(get_mentor).put(update_mentor).delete(delete_mentor),
        )
        .route("/mentors/:id/accept", put(accept_mentor))
        .route("/mentors/:id/reject", put(reject_mentor))
        // Course management
        .route("/courses", get(list_courses).post(create_course))
        .route("/courses/:id", put(update_course).delete(delete_course))
}

// ─── Dashboard Handler ───

/// Admin dashboard with stats
#[utoipa::path(
    get,
    path = "/admin/dashboard",
    tag = "Admin",
    responses(
        (status = 200, description = "Dashboard stats retrieved"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn admin_dashboard(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AuthError> {
    let stats = admin::dashboard(&state.pool).await?;
    Ok((StatusCode::OK, Json(stats)))
}

// ─── Student Handlers ───

/// List all students
#[utoipa::path(
    get,
    path = "/admin/students",
    tag = "Admin",
    responses(
        (status = 200, description = "Students retrieved"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_students(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AuthError> {
    let students = admin::list_students(&state.pool).await?;
    Ok((StatusCode::OK, Json(students)))
}

/// Get single student details
#[utoipa::path(
    get,
    path = "/admin/students/{id}",
    tag = "Admin",
    params(("id" = Uuid, Path, description = "Student UUID")),
    responses(
        (status = 200, description = "Student found"),
        (status = 404, description = "Student not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_student(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    let student = admin::get_student(&state.pool, id).await?;
    Ok((StatusCode::OK, Json(student)))
}

/// Move student to waitlist
#[utoipa::path(
    put,
    path = "/admin/students/{id}/waitlist",
    tag = "Admin",
    params(("id" = Uuid, Path, description = "Student UUID")),
    responses(
        (status = 200, description = "Student waitlisted"),
        (status = 404, description = "Student not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn waitlist_student(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    let student = admin::waitlist_student(&state.pool, id).await?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Student moved to waitlist",
            "student": student
        })),
    ))
}

/// Accept student
#[utoipa::path(
    put,
    path = "/admin/students/{id}/accept",
    tag = "Admin",
    params(("id" = Uuid, Path, description = "Student UUID")),
    responses(
        (status = 200, description = "Student accepted"),
        (status = 404, description = "Student not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn accept_student(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    let student = admin::accept_student(&state.pool, id).await?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Student accepted",
            "student": student
        })),
    ))
}

/// Reject student
#[utoipa::path(
    put,
    path = "/admin/students/{id}/reject",
    tag = "Admin",
    params(("id" = Uuid, Path, description = "Student UUID")),
    responses(
        (status = 200, description = "Student rejected"),
        (status = 404, description = "Student not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn reject_student(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    let student = admin::reject_student(&state.pool, id).await?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Student rejected",
            "student": student
        })),
    ))
}

// ─── Mentor Handlers ───

/// List all mentors
#[utoipa::path(
    get,
    path = "/admin/mentors",
    tag = "Admin",
    responses(
        (status = 200, description = "Mentors retrieved"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_mentors(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AuthError> {
    let mentors = admin::list_mentors(&state.pool).await?;
    Ok((StatusCode::OK, Json(mentors)))
}

/// Get single mentor details
#[utoipa::path(
    get,
    path = "/admin/mentors/{id}",
    tag = "Admin",
    params(("id" = Uuid, Path, description = "Mentor UUID")),
    responses(
        (status = 200, description = "Mentor found"),
        (status = 404, description = "Mentor not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_mentor(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    let mentor = admin::get_mentor(&state.pool, id).await?;
    Ok((StatusCode::OK, Json(mentor)))
}

/// Update mentor details
#[utoipa::path(
    put,
    path = "/admin/mentors/{id}",
    tag = "Admin",
    params(("id" = Uuid, Path, description = "Mentor UUID")),
    responses(
        (status = 200, description = "Mentor updated"),
        (status = 404, description = "Mentor not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_mentor(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateMentorRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let mentor =
        admin::update_mentor(&state.pool, id, req.first_name, req.last_name, req.email).await?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Mentor updated",
            "mentor": mentor
        })),
    ))
}

/// Delete a mentor
#[utoipa::path(
    delete,
    path = "/admin/mentors/{id}",
    tag = "Admin",
    params(("id" = Uuid, Path, description = "Mentor UUID")),
    responses(
        (status = 200, description = "Mentor deleted"),
        (status = 404, description = "Mentor not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_mentor(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    admin::delete_mentor(&state.pool, id).await?;
    Ok((StatusCode::OK, Json(json!({ "message": "Mentor deleted" }))))
}

/// Accept mentor
#[utoipa::path(
    put,
    path = "/admin/mentors/{id}/accept",
    tag = "Admin",
    params(("id" = Uuid, Path, description = "Mentor UUID")),
    responses(
        (status = 200, description = "Mentor accepted"),
        (status = 404, description = "Mentor not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn accept_mentor(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    let mentor = admin::accept_mentor(&state.pool, id).await?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Mentor accepted",
            "mentor": mentor
        })),
    ))
}

/// Reject mentor
#[utoipa::path(
    put,
    path = "/admin/mentors/{id}/reject",
    tag = "Admin",
    params(("id" = Uuid, Path, description = "Mentor UUID")),
    responses(
        (status = 200, description = "Mentor rejected"),
        (status = 404, description = "Mentor not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn reject_mentor(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    let mentor = admin::reject_mentor(&state.pool, id).await?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Mentor rejected",
            "mentor": mentor
        })),
    ))
}

// ─── Course Handlers ───

/// Create a new course
#[utoipa::path(
    post,
    path = "/admin/courses",
    tag = "Admin",
    request_body = CreateCourseRequest,
    responses(
        (status = 201, description = "Course created"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_course(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Json(req): Json<CreateCourseRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let course = admin::create_course(&state.pool, req, admin_user.user_id).await?;
    Ok((StatusCode::CREATED, Json(course)))
}

/// List all courses
#[utoipa::path(
    get,
    path = "/admin/courses",
    tag = "Admin",
    responses(
        (status = 200, description = "Courses retrieved"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_courses(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AuthError> {
    let courses = admin::list_courses(&state.pool).await?;
    Ok((StatusCode::OK, Json(courses)))
}

/// Update a course
#[utoipa::path(
    put,
    path = "/admin/courses/{id}",
    tag = "Admin",
    params(("id" = Uuid, Path, description = "Course UUID")),
    request_body = UpdateCourseRequest,
    responses(
        (status = 200, description = "Course updated"),
        (status = 404, description = "Course not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_course(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCourseRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let course = admin::update_course(&state.pool, id, req).await?;
    Ok((StatusCode::OK, Json(course)))
}

/// Delete a course
#[utoipa::path(
    delete,
    path = "/admin/courses/{id}",
    tag = "Admin",
    params(("id" = Uuid, Path, description = "Course UUID")),
    responses(
        (status = 200, description = "Course deleted"),
        (status = 404, description = "Course not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_course(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    admin::delete_course(&state.pool, id).await?;
    Ok((StatusCode::OK, Json(json!({ "message": "Course deleted" }))))
}

// ─── Application Review Handlers ───

/// List all applications (optionally filter by status)
#[utoipa::path(
    get,
    path = "/admin/applications",
    tag = "Admin",
    params(
        ("status" = Option<String>, Query, description = "Filter by status: pending, accepted, rejected")
    ),
    responses(
        (status = 200, description = "Applications retrieved", body = Vec<ApplicationResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_applications(
    _admin: AdminUser,
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<ApplicationQuery>,
) -> Result<impl IntoResponse, AuthError> {
    let apps = admin::list_applications(&state.pool, query.status.as_deref()).await?;
    Ok((StatusCode::OK, Json(apps)))
}

/// Get a single application by ID
#[utoipa::path(
    get,
    path = "/admin/applications/{id}",
    tag = "Admin",
    params(("id" = Uuid, Path, description = "Application UUID")),
    responses(
        (status = 200, description = "Application found", body = ApplicationResponse),
        (status = 404, description = "Application not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_application(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    let app = admin::get_application(&state.pool, id).await?;
    Ok((StatusCode::OK, Json(app)))
}

/// Accept an application — invites applicant for interview
#[utoipa::path(
    put,
    path = "/admin/applications/{id}/accept",
    tag = "Admin",
    params(("id" = Uuid, Path, description = "Application UUID")),
    responses(
        (status = 200, description = "Applicant invited for interview"),
        (status = 400, description = "Application cannot be transitioned"),
        (status = 404, description = "Application not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn accept_application(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    let application = admin::accept_application(&state, id, admin_user.user_id).await?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Applicant invited for interview",
            "application": application,
        })),
    ))
}

/// Add applicant to the waitlist
#[utoipa::path(
    put,
    path = "/admin/applications/{id}/waitlist",
    tag = "Admin",
    params(("id" = Uuid, Path, description = "Application UUID")),
    responses(
        (status = 200, description = "Applicant added to waitlist"),
        (status = 400, description = "Application cannot be waitlisted"),
        (status = 404, description = "Application not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn waitlist_application(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    let application = admin::waitlist_application(&state, id, admin_user.user_id).await?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Applicant added to waitlist",
            "application": application,
        })),
    ))
}

/// Accept from waitlist — creates user account, enrolls in course
#[utoipa::path(
    put,
    path = "/admin/applications/{id}/enroll",
    tag = "Admin",
    params(("id" = Uuid, Path, description = "Application UUID")),
    responses(
        (status = 200, description = "Account created, enrolled in course"),
        (status = 400, description = "Application not on waitlist"),
        (status = 404, description = "Application not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn enroll_application(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    let (application, temp_password) =
        admin::enroll_application(&state, id, admin_user.user_id).await?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Application accepted — user account created and enrolled",
            "application": application,
            "temp_password": temp_password,
            "note": "An email with login credentials has been sent to the applicant."
        })),
    ))
}

/// Reject an application
#[utoipa::path(
    put,
    path = "/admin/applications/{id}/reject",
    tag = "Admin",
    params(("id" = Uuid, Path, description = "Application UUID")),
    responses(
        (status = 200, description = "Application rejected"),
        (status = 400, description = "Application already accepted or rejected"),
        (status = 404, description = "Application not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn reject_application(
    admin_user: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    let application = admin::reject_application(&state, id, admin_user.user_id).await?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Application rejected",
            "application": application
        })),
    ))
}
