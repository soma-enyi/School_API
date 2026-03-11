use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use sqlx::PgPool;

use crate::controllers::application;
use crate::models::application::ApplicationRequest;
use crate::utils::AuthError;

// ─── Route Definitions ───

/// Public application routes (no auth required)
pub fn application_routes(pool: PgPool) -> Router {
    Router::new()
        .route("/apply", post(submit_application))
        .route("/courses", get(list_available_courses))
        .with_state(pool)
}

// ─── Handlers ───

/// Submit a course application (public — no auth required)
#[utoipa::path(
    post,
    path = "/apply",
    tag = "Applications",
    request_body = ApplicationRequest,
    responses(
        (status = 201, description = "Application submitted successfully", body = ApplicationResponse),
        (status = 400, description = "Invalid request data"),
        (status = 409, description = "Already applied for this course"),
    )
)]
pub async fn submit_application(
    State(pool): State<PgPool>,
    Json(req): Json<ApplicationRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let result = application::submit(&pool, req).await?;
    Ok((StatusCode::CREATED, Json(result)))
}

/// Get list of available courses (public)
#[utoipa::path(
    get,
    path = "/courses",
    tag = "Applications",
    responses(
        (status = 200, description = "List of available courses"),
    )
)]
pub async fn list_available_courses(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, AuthError> {
    let courses = application::list_available_courses(&pool).await?;
    Ok((StatusCode::OK, Json(courses)))
}
