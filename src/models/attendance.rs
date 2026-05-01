use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

// ── Attendance record ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Attendance {
    pub id: Uuid,
    pub course_id: Uuid,
    pub student_id: Uuid,
    pub session_id: Option<Uuid>,
    pub date: NaiveDate,
    pub status: bool,
    pub marked_by: Uuid,
    pub check_in_method: String,
    pub created_at: DateTime<Utc>,
}

/// Mentor manually marks attendance for a student
#[derive(Debug, Deserialize, ToSchema)]
pub struct MarkAttendanceRequest {
    pub course_id: Uuid,
    pub student_id: Uuid,
    pub session_id: Option<Uuid>,
    pub date: NaiveDate,
    pub status: bool,
}

// ── Class session ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ClassSession {
    pub id: Uuid,
    pub course_id: Uuid,
    pub title: String,
    pub session_date: NaiveDate,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSessionRequest {
    pub course_id: Uuid,
    pub title: String,
    pub session_date: NaiveDate,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

// ── QR code ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct QrCode {
    pub id: Uuid,
    pub session_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// ── Self-check-in requests ───────────────────────────────────────────────────

/// Student checks in using their PIN
#[derive(Debug, Deserialize, ToSchema)]
pub struct PinCheckInRequest {
    pub session_id: Uuid,
    pub pin: String,
}

/// Student checks in using a QR token
#[derive(Debug, Deserialize, ToSchema)]
pub struct QrCheckInRequest {
    pub session_id: Uuid,
    pub token: String,
}

/// Student checks in via authenticated login (no extra body needed beyond JWT)
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginCheckInRequest {
    pub session_id: Uuid,
}

/// Set or update the student's PIN
#[derive(Debug, Deserialize, ToSchema)]
pub struct SetPinRequest {
    pub pin: String,
}

// ── Student attendance QR ────────────────────────────────────────────────────

/// QR token generated for a student after they sign in; validated by mentor/admin.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct StudentAttendanceQr {
    pub id: Uuid,
    pub attendance_id: Uuid,
    pub student_id: Uuid,
    pub token: String,
    pub validated: bool,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Mentor/admin submits this token to validate a student's attendance.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ValidateQrRequest {
    pub token: String,
}

// ── Reports ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct AttendanceSummary {
    pub student_id: Uuid,
    pub total: i64,
    pub present: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct SessionAttendanceReport {
    pub session_id: Uuid,
    pub session_title: String,
    pub session_date: NaiveDate,
    pub total_students: i64,
    pub present: i64,
    pub percentage: f64,
}
