use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::middlewares::{AdminUser, MentorUser, StudentUser};
use crate::models::attendance::{
    CreateSessionRequest, LoginCheckInRequest, MarkAttendanceRequest, PinCheckInRequest,
    QrCheckInRequest, SetPinRequest, ValidateQrRequest,
};
use crate::services::attendance_service::AttendanceService;
use crate::utils::AuthError;

pub fn attendance_routes() -> Router<PgPool> {
    Router::new()
        // Session management (admin or mentor)
        .route("/attendance/sessions", post(create_session))
        .route("/attendance/sessions/:course_id", get(list_sessions))
        .route("/attendance/sessions/:session_id/qr", post(generate_qr))
        // Mentor manual mark
        .route("/attendance", post(mark))
        .route("/attendance/:course_id/:date", get(get_by_date))
        // Reports
        .route("/attendance/:course_id/summary", get(get_summary))
        .route("/attendance/:course_id/report", get(session_report))
        // Student self-check-in
        .route("/attendance/checkin/pin", post(checkin_pin))
        .route("/attendance/checkin/qr", post(checkin_qr))
        .route("/attendance/checkin/login", post(checkin_login))
        .route("/attendance/student/:student_id", get(get_by_student))
        // PIN management
        .route("/attendance/pin", put(set_pin))
        // Student attendance QR validation
        .route("/attendance/my-qr/:attendance_id", get(get_my_qr))
        .route("/attendance/validate-qr", post(validate_qr))
}

// ── Session management ───────────────────────────────────────────────────────

/// Create a class session (mentor only)
#[utoipa::path(
    post,
    path = "/attendance/sessions",
    tag = "Attendance",
    request_body = CreateSessionRequest,
    responses(
        (status = 201, description = "Session created", body = crate::models::attendance::ClassSession),
        (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_session(
    mentor: MentorUser,
    State(pool): State<PgPool>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let session = AttendanceService::create_session(&pool, req, mentor.user_id).await?;
    Ok((StatusCode::CREATED, Json(session)))
}

/// List sessions for a course (mentor only)
#[utoipa::path(
    get,
    path = "/attendance/sessions/{course_id}",
    tag = "Attendance",
    params(("course_id" = Uuid, Path, description = "Course UUID")),
    responses(
        (status = 200, description = "Sessions list", body = Vec<crate::models::attendance::ClassSession>),
        (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_sessions(
    _mentor: MentorUser,
    State(pool): State<PgPool>,
    Path(course_id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    let sessions = AttendanceService::list_sessions(&pool, course_id).await?;
    Ok((StatusCode::OK, Json(sessions)))
}

/// Generate QR code for a session (mentor only)
#[utoipa::path(
    post,
    path = "/attendance/sessions/{session_id}/qr",
    tag = "Attendance",
    params(("session_id" = Uuid, Path, description = "Session UUID")),
    responses(
        (status = 201, description = "QR code generated", body = crate::models::attendance::QrCode),
        (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn generate_qr(
    _mentor: MentorUser,
    State(pool): State<PgPool>,
    Path(session_id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    let qr = AttendanceService::generate_qr(&pool, session_id).await?;
    Ok((StatusCode::CREATED, Json(qr)))
}

// ── Mentor manual mark ───────────────────────────────────────────────────────

/// Manually mark attendance for a student (mentor only)
#[utoipa::path(
    post,
    path = "/attendance",
    tag = "Attendance",
    request_body = MarkAttendanceRequest,
    responses(
        (status = 201, description = "Attendance marked", body = crate::models::attendance::Attendance),
        (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn mark(
    mentor: MentorUser,
    State(pool): State<PgPool>,
    Json(req): Json<MarkAttendanceRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let record = AttendanceService::mark(&pool, req, mentor.user_id).await?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// Get attendance for a course on a specific date (mentor only)
#[utoipa::path(
    get,
    path = "/attendance/{course_id}/{date}",
    tag = "Attendance",
    params(
        ("course_id" = Uuid, Path, description = "Course UUID"),
        ("date" = String, Path, description = "Date YYYY-MM-DD"),
    ),
    responses(
        (status = 200, description = "Attendance records", body = Vec<crate::models::attendance::Attendance>),
        (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_by_date(
    _mentor: MentorUser,
    State(pool): State<PgPool>,
    Path((course_id, date)): Path<(Uuid, chrono::NaiveDate)>,
) -> Result<impl IntoResponse, AuthError> {
    let records = AttendanceService::get_by_date(&pool, course_id, date).await?;
    Ok((StatusCode::OK, Json(records)))
}

// ── Reports ──────────────────────────────────────────────────────────────────

/// Attendance summary per student for a course (mentor only)
#[utoipa::path(
    get,
    path = "/attendance/{course_id}/summary",
    tag = "Attendance",
    params(("course_id" = Uuid, Path, description = "Course UUID")),
    responses(
        (status = 200, description = "Attendance summary", body = Vec<crate::models::attendance::AttendanceSummary>),
        (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_summary(
    _mentor: MentorUser,
    State(pool): State<PgPool>,
    Path(course_id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    let summary = AttendanceService::get_summary(&pool, course_id).await?;
    Ok((StatusCode::OK, Json(summary)))
}

/// Per-session attendance report for a course (mentor only)
#[utoipa::path(
    get,
    path = "/attendance/{course_id}/report",
    tag = "Attendance",
    params(("course_id" = Uuid, Path, description = "Course UUID")),
    responses(
        (status = 200, description = "Session attendance report", body = Vec<crate::models::attendance::SessionAttendanceReport>),
        (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn session_report(
    _mentor: MentorUser,
    State(pool): State<PgPool>,
    Path(course_id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    let report = AttendanceService::get_session_report(&pool, course_id).await?;
    Ok((StatusCode::OK, Json(report)))
}

// ── Student self-check-in ────────────────────────────────────────────────────

/// Student checks in using PIN
#[utoipa::path(
    post,
    path = "/attendance/checkin/pin",
    tag = "Attendance",
    request_body = PinCheckInRequest,
    responses(
        (status = 201, description = "Checked in", body = crate::models::attendance::Attendance),
        (status = 401, description = "Invalid PIN", body = crate::utils::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn checkin_pin(
    student: StudentUser,
    State(pool): State<PgPool>,
    Json(req): Json<PinCheckInRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let record = AttendanceService::checkin_pin(&pool, req.session_id, student.user_id, &req.pin).await?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// Student checks in using QR token
#[utoipa::path(
    post,
    path = "/attendance/checkin/qr",
    tag = "Attendance",
    request_body = QrCheckInRequest,
    responses(
        (status = 201, description = "Checked in", body = crate::models::attendance::Attendance),
        (status = 401, description = "Invalid or expired QR token", body = crate::utils::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn checkin_qr(
    student: StudentUser,
    State(pool): State<PgPool>,
    Json(req): Json<QrCheckInRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let record = AttendanceService::checkin_qr(&pool, req.session_id, student.user_id, &req.token).await?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// Student checks in via authenticated login
#[utoipa::path(
    post,
    path = "/attendance/checkin/login",
    tag = "Attendance",
    request_body = LoginCheckInRequest,
    responses(
        (status = 201, description = "Checked in", body = crate::models::attendance::Attendance),
        (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn checkin_login(
    student: StudentUser,
    State(pool): State<PgPool>,
    Json(req): Json<LoginCheckInRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let record = AttendanceService::checkin_login(&pool, req.session_id, student.user_id).await?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// Get student's own attendance history
#[utoipa::path(
    get,
    path = "/attendance/student/{student_id}",
    tag = "Attendance",
    params(("student_id" = Uuid, Path, description = "Student UUID")),
    responses(
        (status = 200, description = "Attendance history", body = Vec<crate::models::attendance::Attendance>),
        (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_by_student(
    student: StudentUser,
    State(pool): State<PgPool>,
    Path(student_id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    if student.user_id != student_id {
        return Err(AuthError::Forbidden);
    }
    let records = AttendanceService::get_by_student(&pool, student_id).await?;
    Ok((StatusCode::OK, Json(records)))
}

// ── PIN management ───────────────────────────────────────────────────────────

/// Set or update student's check-in PIN
#[utoipa::path(
    put,
    path = "/attendance/pin",
    tag = "Attendance",
    request_body = SetPinRequest,
    responses(
        (status = 200, description = "PIN updated"),
        (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn set_pin(
    student: StudentUser,
    State(pool): State<PgPool>,
    Json(req): Json<SetPinRequest>,
) -> Result<impl IntoResponse, AuthError> {
    AttendanceService::set_pin(&pool, student.user_id, &req.pin).await?;
    Ok(StatusCode::OK)
}

// ── Student attendance QR ────────────────────────────────────────────────────

/// Get the QR token for a student's attendance record (student only, own record)
#[utoipa::path(
    get,
    path = "/attendance/my-qr/{attendance_id}",
    tag = "Attendance",
    params(("attendance_id" = Uuid, Path, description = "Attendance record UUID")),
    responses(
        (status = 200, description = "QR token", body = crate::models::attendance::StudentAttendanceQr),
        (status = 401, description = "Unauthorized", body = crate::utils::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_my_qr(
    student: StudentUser,
    State(pool): State<PgPool>,
    Path(attendance_id): Path<Uuid>,
) -> Result<impl IntoResponse, AuthError> {
    let qr = sqlx::query_as::<_, crate::models::attendance::StudentAttendanceQr>(
        "SELECT * FROM student_attendance_qr WHERE attendance_id = $1 AND student_id = $2",
    )
    .bind(attendance_id)
    .bind(student.user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?
    .ok_or(AuthError::UserNotFound)?;

    Ok((StatusCode::OK, Json(qr)))
}

/// Validate a student's attendance QR token (mentor or admin only) — sets status = true
#[utoipa::path(
    post,
    path = "/attendance/validate-qr",
    tag = "Attendance",
    request_body = ValidateQrRequest,
    responses(
        (status = 200, description = "Attendance validated", body = crate::models::attendance::Attendance),
        (status = 401, description = "Invalid or expired token", body = crate::utils::ErrorResponse),
        (status = 403, description = "Forbidden", body = crate::utils::ErrorResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn validate_qr(
    _mentor: MentorUser,
    State(pool): State<PgPool>,
    Json(req): Json<ValidateQrRequest>,
) -> Result<impl IntoResponse, AuthError> {
    let attendance = AttendanceService::validate_attendance_qr(&pool, &req.token).await?;
    Ok((StatusCode::OK, Json(attendance)))
}
