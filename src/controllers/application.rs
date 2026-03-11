use sqlx::PgPool;

use crate::models::application::{ApplicationRequest, ApplicationResponse};
use crate::models::CourseResponse;
use crate::services::{ApplicationService, CourseService};
use crate::utils::AuthError;

/// Submit a new course application (public).
pub async fn submit(
    pool: &PgPool,
    req: ApplicationRequest,
) -> Result<ApplicationResponse, AuthError> {
    ApplicationService::submit(pool, req).await
}

/// List all available courses (public).
pub async fn list_available_courses(
    pool: &PgPool,
) -> Result<Vec<CourseResponse>, AuthError> {
    CourseService::list_courses(pool).await
}
