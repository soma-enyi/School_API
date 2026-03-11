use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::application::ApplicationResponse;
use crate::models::course::{CourseResponse, CreateCourseRequest, UpdateCourseRequest};
use crate::models::UserResponse;
use crate::services::{AdminService, ApplicationService, CourseService};
use crate::state::AppState;
use crate::utils::AuthError;

// ─── Dashboard ───

pub async fn dashboard(pool: &PgPool) -> Result<Value, AuthError> {
    AdminService::get_dashboard_stats(pool).await
}

// ─── Student Management ───

pub async fn list_students(pool: &PgPool) -> Result<Vec<UserResponse>, AuthError> {
    AdminService::list_students(pool).await
}

pub async fn get_student(pool: &PgPool, id: Uuid) -> Result<UserResponse, AuthError> {
    AdminService::get_student(pool, id).await
}

pub async fn waitlist_student(pool: &PgPool, id: Uuid) -> Result<UserResponse, AuthError> {
    AdminService::waitlist_student(pool, id).await
}

pub async fn accept_student(pool: &PgPool, id: Uuid) -> Result<UserResponse, AuthError> {
    AdminService::accept_student(pool, id).await
}

pub async fn reject_student(pool: &PgPool, id: Uuid) -> Result<UserResponse, AuthError> {
    AdminService::reject_student(pool, id).await
}

// ─── Mentor Management ───

pub async fn list_mentors(pool: &PgPool) -> Result<Vec<UserResponse>, AuthError> {
    AdminService::list_mentors(pool).await
}

pub async fn get_mentor(pool: &PgPool, id: Uuid) -> Result<UserResponse, AuthError> {
    AdminService::get_mentor(pool, id).await
}

pub async fn update_mentor(
    pool: &PgPool,
    id: Uuid,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
) -> Result<UserResponse, AuthError> {
    AdminService::update_mentor(pool, id, first_name, last_name, email).await
}

pub async fn delete_mentor(pool: &PgPool, id: Uuid) -> Result<(), AuthError> {
    AdminService::delete_mentor(pool, id).await
}

pub async fn accept_mentor(pool: &PgPool, id: Uuid) -> Result<UserResponse, AuthError> {
    AdminService::accept_mentor(pool, id).await
}

pub async fn reject_mentor(pool: &PgPool, id: Uuid) -> Result<UserResponse, AuthError> {
    AdminService::reject_mentor(pool, id).await
}

// ─── Course Management ───

pub async fn create_course(
    pool: &PgPool,
    req: CreateCourseRequest,
    admin_id: Uuid,
) -> Result<CourseResponse, AuthError> {
    CourseService::create_course(pool, req, admin_id).await
}

pub async fn list_courses(pool: &PgPool) -> Result<Vec<CourseResponse>, AuthError> {
    CourseService::list_courses(pool).await
}

pub async fn update_course(
    pool: &PgPool,
    id: Uuid,
    req: UpdateCourseRequest,
) -> Result<CourseResponse, AuthError> {
    CourseService::update_course(pool, id, req).await
}

pub async fn delete_course(pool: &PgPool, id: Uuid) -> Result<(), AuthError> {
    CourseService::delete_course(pool, id).await
}

// ─── Application Review ───

pub async fn list_applications(
    pool: &PgPool,
    status: Option<&str>,
) -> Result<Vec<ApplicationResponse>, AuthError> {
    ApplicationService::list(pool, status).await
}

pub async fn get_application(pool: &PgPool, id: Uuid) -> Result<ApplicationResponse, AuthError> {
    ApplicationService::get(pool, id).await
}

/// Accept application — invites applicant for interview, sends email
pub async fn accept_application(
    state: &AppState,
    id: Uuid,
    admin_id: Uuid,
) -> Result<ApplicationResponse, AuthError> {
    let application =
        ApplicationService::invite_for_interview(&state.pool, id, admin_id).await?;

    // Send interview invitation email with venue
    let applicant_name = format!("{} {}", application.first_name, application.last_name);
    let course_name = application.course_name.clone().unwrap_or_default();
    let email = application.email.clone();
    let interview_venue = std::env::var("INTERVIEW_VENUE")
        .unwrap_or_else(|_| "CourseFlow HQ — Details will be sent via email".to_string());
    state
        .try_send_email("interview invitation", |svc| async move {
            svc.send_interview_invitation(&email, &applicant_name, &course_name, &interview_venue)
                .await
        })
        .await;

    Ok(application)
}

/// Waitlist application — no email sent
pub async fn waitlist_application(
    state: &AppState,
    id: Uuid,
    admin_id: Uuid,
) -> Result<ApplicationResponse, AuthError> {
    ApplicationService::add_to_waitlist(&state.pool, id, admin_id).await
}

/// Enroll application — creates user account, sends enrollment email, returns temp password
pub async fn enroll_application(
    state: &AppState,
    id: Uuid,
    admin_id: Uuid,
) -> Result<(ApplicationResponse, String), AuthError> {
    let (application, temp_password) =
        ApplicationService::accept_from_waitlist(&state.pool, id, admin_id).await?;

    // Send enrollment acceptance email with temp password
    let applicant_name = format!("{} {}", application.first_name, application.last_name);
    let course_name = application.course_name.clone().unwrap_or_default();
    let email = application.email.clone();
    let password = temp_password.clone();
    state
        .try_send_email("enrollment acceptance", |svc| async move {
            svc.send_enrollment_acceptance(&email, &applicant_name, &course_name, &password)
                .await
        })
        .await;

    Ok((application, temp_password))
}

/// Reject application — sends rejection email
pub async fn reject_application(
    state: &AppState,
    id: Uuid,
    admin_id: Uuid,
) -> Result<ApplicationResponse, AuthError> {
    let application = ApplicationService::reject(&state.pool, id, admin_id).await?;

    // Send rejection email
    let applicant_name = format!("{} {}", application.first_name, application.last_name);
    let course_name = application.course_name.clone().unwrap_or_default();
    let email = application.email.clone();
    state
        .try_send_email("rejection notification", |svc| async move {
            svc.send_rejection_email(&email, &applicant_name, &course_name)
                .await
        })
        .await;

    Ok(application)
}
