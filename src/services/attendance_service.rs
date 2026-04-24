use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Timelike, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::attendance::{
    Attendance, AttendanceSummary, ClassSession, CreateSessionRequest, MarkAttendanceRequest,
    QrCode, SessionAttendanceReport, StudentAttendanceQr,
};
use crate::utils::AuthError;

pub struct AttendanceService;

impl AttendanceService {
    /// Enforce the 8:00 AM – 10:14 AM sign-in window (UTC).
    fn check_signin_window() -> Result<(), AuthError> {
        let now = Utc::now();
        let (h, m) = (now.hour(), now.minute());
        let minutes = h * 60 + m;
        if minutes < 8 * 60 {
            return Err(AuthError::SignInNotOpen);
        }
        if minutes >= 10 * 60 + 15 {
            return Err(AuthError::SignInClosed);
        }
        Ok(())
    }
    // ── Class sessions ───────────────────────────────────────────────────────

    pub async fn create_session(
        pool: &PgPool,
        req: CreateSessionRequest,
        created_by: Uuid,
    ) -> Result<ClassSession, AuthError> {
        sqlx::query_as::<_, ClassSession>(
            "INSERT INTO class_sessions (course_id, title, session_date, starts_at, ends_at, created_by)
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
        )
        .bind(req.course_id)
        .bind(req.title)
        .bind(req.session_date)
        .bind(req.starts_at)
        .bind(req.ends_at)
        .bind(created_by)
        .fetch_one(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))
    }

    pub async fn list_sessions(
        pool: &PgPool,
        course_id: Uuid,
    ) -> Result<Vec<ClassSession>, AuthError> {
        sqlx::query_as::<_, ClassSession>(
            "SELECT * FROM class_sessions WHERE course_id = $1 ORDER BY starts_at DESC",
        )
        .bind(course_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))
    }

    // ── QR codes ─────────────────────────────────────────────────────────────

    /// Generate (or replace) a QR token for a session. Token expires when session ends.
    pub async fn generate_qr(pool: &PgPool, session_id: Uuid) -> Result<QrCode, AuthError> {
        let session = sqlx::query_as::<_, ClassSession>(
            "SELECT * FROM class_sessions WHERE id = $1",
        )
        .bind(session_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::UserNotFound)?;

        let token = Uuid::new_v4().to_string();

        sqlx::query_as::<_, QrCode>(
            "INSERT INTO qr_codes (session_id, token, expires_at)
             VALUES ($1, $2, $3)
             ON CONFLICT (session_id)
             DO UPDATE SET token = EXCLUDED.token, expires_at = EXCLUDED.expires_at
             RETURNING *",
        )
        .bind(session_id)
        .bind(&token)
        .bind(session.ends_at)
        .fetch_one(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))
    }

    // ── PIN management ───────────────────────────────────────────────────────

    pub async fn set_pin(pool: &PgPool, student_id: Uuid, pin: &str) -> Result<(), AuthError> {
        let pin_hash = hash(pin, DEFAULT_COST)
            .map_err(|_| AuthError::InternalServerError)?;

        sqlx::query(
            "INSERT INTO student_pins (student_id, pin_hash, updated_at)
             VALUES ($1, $2, NOW())
             ON CONFLICT (student_id)
             DO UPDATE SET pin_hash = EXCLUDED.pin_hash, updated_at = NOW()",
        )
        .bind(student_id)
        .bind(pin_hash)
        .execute(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // ── Self-check-in helpers ────────────────────────────────────────────────

    /// Shared logic: record attendance for a student in a session, preventing duplicates.
    /// Also auto-generates a student attendance QR token after a successful check-in.
    async fn record_checkin(
        pool: &PgPool,
        session_id: Uuid,
        student_id: Uuid,
        method: &str,
    ) -> Result<Attendance, AuthError> {
        // Fetch session to get course_id and date
        let session = sqlx::query_as::<_, ClassSession>(
            "SELECT * FROM class_sessions WHERE id = $1",
        )
        .bind(session_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::UserNotFound)?;

        // Prevent duplicate check-in (unique constraint on course_id + student_id + date)
        let attendance = sqlx::query_as::<_, Attendance>(
            "INSERT INTO attendance
                (course_id, student_id, session_id, date, status, marked_by, check_in_method)
             VALUES ($1, $2, $3, $4, TRUE, $2, $5)
             ON CONFLICT (course_id, student_id, date) DO NOTHING
             RETURNING *",
        )
        .bind(session.course_id)
        .bind(student_id)
        .bind(session_id)
        .bind(session.session_date)
        .bind(method)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AuthError::DatabaseError("Already checked in for this session".into()))?;

        // Auto-generate a unique QR token for this attendance record
        Self::generate_attendance_qr(pool, attendance.id, student_id).await?;

        Ok(attendance)
    }

    /// Check in via PIN
    pub async fn checkin_pin(
        pool: &PgPool,
        session_id: Uuid,
        student_id: Uuid,
        pin: &str,
    ) -> Result<Attendance, AuthError> {
        Self::check_signin_window()?;

        let row = sqlx::query_scalar::<_, String>(
            "SELECT pin_hash FROM student_pins WHERE student_id = $1",
        )
        .bind(student_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::InvalidCredentials)?;

        if !verify(pin, &row).map_err(|_| AuthError::InternalServerError)? {
            return Err(AuthError::InvalidCredentials);
        }

        Self::record_checkin(pool, session_id, student_id, "pin").await
    }

    /// Check in via QR token
    pub async fn checkin_qr(
        pool: &PgPool,
        session_id: Uuid,
        student_id: Uuid,
        token: &str,
    ) -> Result<Attendance, AuthError> {
        Self::check_signin_window()?;

        let qr = sqlx::query_as::<_, QrCode>(
            "SELECT * FROM qr_codes WHERE session_id = $1 AND token = $2",
        )
        .bind(session_id)
        .bind(token)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::InvalidToken)?;

        if qr.expires_at < Utc::now() {
            return Err(AuthError::TokenExpired);
        }

        Self::record_checkin(pool, session_id, student_id, "qr").await
    }

    /// Check in via authenticated login (JWT already verified by extractor)
    pub async fn checkin_login(
        pool: &PgPool,
        session_id: Uuid,
        student_id: Uuid,
    ) -> Result<Attendance, AuthError> {
        Self::check_signin_window()?;
        Self::record_checkin(pool, session_id, student_id, "login").await
    }

    // ── Mentor manual mark ───────────────────────────────────────────────────

    pub async fn mark(
        pool: &PgPool,
        req: MarkAttendanceRequest,
        marked_by: Uuid,
    ) -> Result<Attendance, AuthError> {
        sqlx::query_as::<_, Attendance>(
            "INSERT INTO attendance
                (course_id, student_id, session_id, date, status, marked_by, check_in_method)
             VALUES ($1, $2, $3, $4, $5, $6, 'manual')
             ON CONFLICT (course_id, student_id, date)
             DO UPDATE SET status = EXCLUDED.status, marked_by = EXCLUDED.marked_by
             RETURNING *",
        )
        .bind(req.course_id)
        .bind(req.student_id)
        .bind(req.session_id)
        .bind(req.date)
        .bind(req.status)
        .bind(marked_by)
        .fetch_one(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))
    }

    // ── Queries ──────────────────────────────────────────────────────────────

    pub async fn get_by_date(
        pool: &PgPool,
        course_id: Uuid,
        date: chrono::NaiveDate,
    ) -> Result<Vec<Attendance>, AuthError> {
        sqlx::query_as::<_, Attendance>(
            "SELECT * FROM attendance WHERE course_id = $1 AND date = $2 ORDER BY student_id",
        )
        .bind(course_id)
        .bind(date)
        .fetch_all(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))
    }

    pub async fn get_by_student(
        pool: &PgPool,
        student_id: Uuid,
    ) -> Result<Vec<Attendance>, AuthError> {
        sqlx::query_as::<_, Attendance>(
            "SELECT * FROM attendance WHERE student_id = $1 ORDER BY date DESC",
        )
        .bind(student_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))
    }

    /// Attendance summary per student for a course, including percentage
    pub async fn get_summary(
        pool: &PgPool,
        course_id: Uuid,
    ) -> Result<Vec<AttendanceSummary>, AuthError> {
        sqlx::query_as::<_, AttendanceSummary>(
            "SELECT student_id,
                    COUNT(*) AS total,
                    SUM(CASE WHEN status THEN 1 ELSE 0 END) AS present,
                    ROUND(
                        100.0 * SUM(CASE WHEN status THEN 1 ELSE 0 END) / NULLIF(COUNT(*), 0),
                        2
                    ) AS percentage
             FROM attendance
             WHERE course_id = $1
             GROUP BY student_id",
        )
        .bind(course_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))
    }

    /// Per-session report for a course
    pub async fn get_session_report(
        pool: &PgPool,
        course_id: Uuid,
    ) -> Result<Vec<SessionAttendanceReport>, AuthError> {
        sqlx::query_as::<_, SessionAttendanceReport>(
            "SELECT cs.id AS session_id,
                    cs.title AS session_title,
                    cs.session_date,
                    COUNT(a.id) AS total_students,
                    SUM(CASE WHEN a.status THEN 1 ELSE 0 END) AS present,
                    ROUND(
                        100.0 * SUM(CASE WHEN a.status THEN 1 ELSE 0 END) / NULLIF(COUNT(a.id), 0),
                        2
                    ) AS percentage
             FROM class_sessions cs
             LEFT JOIN attendance a ON a.session_id = cs.id
             WHERE cs.course_id = $1
             GROUP BY cs.id, cs.title, cs.session_date
             ORDER BY cs.session_date DESC",
        )
        .bind(course_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))
    }

    // ── Student attendance QR ────────────────────────────────────────────────

    /// Generate a unique QR token for a student's attendance record.
    /// Called automatically after a successful sign-in.
    /// Token expires in 24 hours.
    pub async fn generate_attendance_qr(
        pool: &PgPool,
        attendance_id: Uuid,
        student_id: Uuid,
    ) -> Result<StudentAttendanceQr, AuthError> {
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + chrono::Duration::hours(24);

        sqlx::query_as::<_, StudentAttendanceQr>(
            "INSERT INTO student_attendance_qr (attendance_id, student_id, token, expires_at)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (attendance_id)
             DO UPDATE SET token = EXCLUDED.token, expires_at = EXCLUDED.expires_at, validated = FALSE
             RETURNING *",
        )
        .bind(attendance_id)
        .bind(student_id)
        .bind(token)
        .bind(expires_at)
        .fetch_one(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))
    }

    /// Validate a student's QR token (mentor/admin only).
    /// Sets attendance.status = true and marks the QR as validated.
    pub async fn validate_attendance_qr(
        pool: &PgPool,
        token: &str,
    ) -> Result<Attendance, AuthError> {
        let qr = sqlx::query_as::<_, StudentAttendanceQr>(
            "SELECT * FROM student_attendance_qr WHERE token = $1",
        )
        .bind(token)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::InvalidToken)?;

        if qr.expires_at < Utc::now() {
            return Err(AuthError::TokenExpired);
        }
        if qr.validated {
            return Err(AuthError::DatabaseError("QR token already validated".into()));
        }

        // Mark QR as validated
        sqlx::query("UPDATE student_attendance_qr SET validated = TRUE WHERE id = $1")
            .bind(qr.id)
            .execute(pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        // Set attendance status = true
        sqlx::query_as::<_, Attendance>(
            "UPDATE attendance SET status = TRUE WHERE id = $1 RETURNING *",
        )
        .bind(qr.attendance_id)
        .fetch_one(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))
    }
}
