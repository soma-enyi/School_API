use sqlx::PgPool;
use uuid::Uuid;

use crate::models::application::{Application, ApplicationRequest, ApplicationResponse};
use crate::utils::AuthError;

/// Row returned when joining applications with courses
#[derive(sqlx::FromRow)]
struct ApplicationWithCourse {
    id: Uuid,
    first_name: String,
    last_name: String,
    email: String,
    course_id: Uuid,
    motivation: String,
    experience: Option<String>,
    status: String,
    reviewed_by: Option<Uuid>,
    reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    course_name: Option<String>,
}

impl From<ApplicationWithCourse> for ApplicationResponse {
    fn from(a: ApplicationWithCourse) -> Self {
        ApplicationResponse {
            id: a.id,
            first_name: a.first_name,
            last_name: a.last_name,
            email: a.email,
            course_id: a.course_id,
            course_name: a.course_name,
            motivation: a.motivation,
            experience: a.experience,
            status: a.status,
            reviewed_by: a.reviewed_by,
            reviewed_at: a.reviewed_at,
            created_at: a.created_at,
        }
    }
}

pub struct ApplicationService;

impl ApplicationService {
    // ─── Public ───

    /// Submit a new application (public, no auth)
    pub async fn submit(
        pool: &PgPool,
        req: ApplicationRequest,
    ) -> Result<ApplicationResponse, AuthError> {
        // Verify the course exists
        let course_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM courses WHERE id = $1)"
        )
        .bind(req.course_id)
        .fetch_one(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        if !course_exists {
            return Err(AuthError::DatabaseError("Course not found".into()));
        }

        let app = sqlx::query_as::<_, Application>(
            "INSERT INTO applications (first_name, last_name, email, course_id, motivation, experience)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, first_name, last_name, email, course_id, motivation, experience, status, reviewed_by, reviewed_at, created_at"
        )
        .bind(&req.first_name)
        .bind(&req.last_name)
        .bind(&req.email)
        .bind(req.course_id)
        .bind(&req.motivation)
        .bind(&req.experience)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate key") {
                AuthError::UserAlreadyExists // reuse: "already applied for this course"
            } else {
                AuthError::DatabaseError(e.to_string())
            }
        })?;

        // Fetch course name
        let course_name = sqlx::query_scalar::<_, String>(
            "SELECT name FROM courses WHERE id = $1"
        )
        .bind(app.course_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        let mut resp = ApplicationResponse::from(app);
        resp.course_name = course_name;
        Ok(resp)
    }

    // ─── Admin ───

    /// List all applications, optionally filtered by status
    pub async fn list(
        pool: &PgPool,
        status_filter: Option<&str>,
    ) -> Result<Vec<ApplicationResponse>, AuthError> {
        let rows = if let Some(status) = status_filter {
            sqlx::query_as::<_, ApplicationWithCourse>(
                "SELECT a.id, a.first_name, a.last_name, a.email, a.course_id,
                        a.motivation, a.experience, a.status, a.reviewed_by,
                        a.reviewed_at, a.created_at,
                        c.name AS course_name
                 FROM applications a
                 JOIN courses c ON c.id = a.course_id
                 WHERE a.status = $1
                 ORDER BY a.created_at DESC"
            )
            .bind(status)
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_as::<_, ApplicationWithCourse>(
                "SELECT a.id, a.first_name, a.last_name, a.email, a.course_id,
                        a.motivation, a.experience, a.status, a.reviewed_by,
                        a.reviewed_at, a.created_at,
                        c.name AS course_name
                 FROM applications a
                 JOIN courses c ON c.id = a.course_id
                 ORDER BY a.created_at DESC"
            )
            .fetch_all(pool)
            .await
        }
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(ApplicationResponse::from).collect())
    }

    /// Get a single application by ID
    pub async fn get(pool: &PgPool, id: Uuid) -> Result<ApplicationResponse, AuthError> {
        let row = sqlx::query_as::<_, ApplicationWithCourse>(
            "SELECT a.id, a.first_name, a.last_name, a.email, a.course_id,
                    a.motivation, a.experience, a.status, a.reviewed_by,
                    a.reviewed_at, a.created_at,
                    c.name AS course_name
             FROM applications a
             JOIN courses c ON c.id = a.course_id
             WHERE a.id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::UserNotFound)?;

        Ok(ApplicationResponse::from(row))
    }

    // ─── Pipeline step 1: Invite for interview ───

    /// Move application from `pending` → `interview_invited`.
    /// No account is created — the admin just invites the applicant for an
    /// interview (an email would be sent by the controller layer).
    pub async fn invite_for_interview(
        pool: &PgPool,
        application_id: Uuid,
        admin_id: Uuid,
    ) -> Result<ApplicationResponse, AuthError> {
        let app = Self::fetch_raw(pool, application_id).await?;

        if app.status != "pending" {
            return Err(AuthError::DatabaseError(
                format!("Cannot invite: application is already '{}'", app.status),
            ));
        }

        let updated = Self::transition(pool, application_id, admin_id, "interview_invited").await?;
        Ok(ApplicationResponse::from(updated))
    }

    // ─── Pipeline step 2: Add to waitlist ───

    /// Move application from `interview_invited` → `waitlisted`.
    /// Called after the applicant passes the interview.
    pub async fn add_to_waitlist(
        pool: &PgPool,
        application_id: Uuid,
        admin_id: Uuid,
    ) -> Result<ApplicationResponse, AuthError> {
        let app = Self::fetch_raw(pool, application_id).await?;

        if app.status != "interview_invited" {
            return Err(AuthError::DatabaseError(
                format!("Cannot waitlist: application status is '{}', expected 'interview_invited'", app.status),
            ));
        }

        let updated = Self::transition(pool, application_id, admin_id, "waitlisted").await?;
        Ok(ApplicationResponse::from(updated))
    }

    // ─── Pipeline step 3: Accept from waitlist (creates account) ───

    /// Move application from `waitlisted` → `accepted`.
    ///  1. Create a user account (student role, random temp password)
    ///  2. Enroll the user in the course
    ///  3. Return the application + temp password (for the admin to email)
    pub async fn accept_from_waitlist(
        pool: &PgPool,
        application_id: Uuid,
        admin_id: Uuid,
    ) -> Result<(ApplicationResponse, String), AuthError> {
        let app = Self::fetch_raw(pool, application_id).await?;

        if app.status != "waitlisted" {
            return Err(AuthError::DatabaseError(
                format!("Cannot accept: application status is '{}', expected 'waitlisted'", app.status),
            ));
        }

        // Generate a random temporary password
        use rand::Rng;
        let temp_password: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(12)
            .map(char::from)
            .collect();

        let password_hash = bcrypt::hash(&temp_password, bcrypt::DEFAULT_COST)
            .map_err(|_| AuthError::InternalServerError)?;

        // Check if a user with this email already exists
        let existing = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"
        )
        .bind(&app.email)
        .fetch_one(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        let user_id: Uuid = if existing {
            sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE email = $1")
                .bind(&app.email)
                .fetch_one(pool)
                .await
                .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query_scalar::<_, Uuid>(
                "INSERT INTO users (email, password_hash, first_name, last_name, role, is_active, status)
                 VALUES ($1, $2, $3, $4, 'student', true, 'accepted')
                 RETURNING id"
            )
            .bind(&app.email)
            .bind(&password_hash)
            .bind(&app.first_name)
            .bind(&app.last_name)
            .fetch_one(pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        };

        // Enroll user in the course
        sqlx::query(
            "INSERT INTO course_enrollments (user_id, course_id, role)
             VALUES ($1, $2, 'student')
             ON CONFLICT (user_id, course_id) DO NOTHING"
        )
        .bind(user_id)
        .bind(app.course_id)
        .execute(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        // Mark accepted
        let updated = Self::transition(pool, application_id, admin_id, "accepted").await?;
        Ok((ApplicationResponse::from(updated), temp_password))
    }

    // ─── Reject (from any non-accepted stage) ───

    /// Reject an application. Allowed from any status except `accepted`.
    pub async fn reject(
        pool: &PgPool,
        application_id: Uuid,
        admin_id: Uuid,
    ) -> Result<ApplicationResponse, AuthError> {
        let app = Self::fetch_raw(pool, application_id).await?;

        if app.status == "accepted" || app.status == "rejected" {
            return Err(AuthError::DatabaseError(
                format!("Cannot reject: application is already '{}'", app.status),
            ));
        }

        let updated = Self::transition(pool, application_id, admin_id, "rejected").await?;
        Ok(ApplicationResponse::from(updated))
    }

    // ─── Helpers ───

    /// Fetch the raw Application row (no join)
    async fn fetch_raw(pool: &PgPool, id: Uuid) -> Result<Application, AuthError> {
        sqlx::query_as::<_, Application>(
            "SELECT id, first_name, last_name, email, course_id, motivation, experience,
                    status, reviewed_by, reviewed_at, created_at
             FROM applications WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::UserNotFound)
    }

    /// Transition an application to a new status and return it with course name
    async fn transition(
        pool: &PgPool,
        application_id: Uuid,
        admin_id: Uuid,
        new_status: &str,
    ) -> Result<ApplicationWithCourse, AuthError> {
        sqlx::query_as::<_, ApplicationWithCourse>(
            "UPDATE applications
             SET status = $3, reviewed_by = $2, reviewed_at = NOW()
             WHERE id = $1
             RETURNING id, first_name, last_name, email, course_id, motivation, experience,
                       status, reviewed_by, reviewed_at, created_at,
                       (SELECT name FROM courses WHERE id = applications.course_id) AS course_name"
        )
        .bind(application_id)
        .bind(admin_id)
        .bind(new_status)
        .fetch_one(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))
    }
}
