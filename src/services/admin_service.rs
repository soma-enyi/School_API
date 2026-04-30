use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::{User, UserResponse};
use crate::utils::AuthError;

pub struct AdminService;

impl AdminService {
    // ─── Student Management ───

    /// List all students
    pub async fn list_students(pool: &PgPool) -> Result<Vec<UserResponse>, AuthError> {
        let users = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, first_name, last_name, role, is_active, created_at, updated_at, status
             FROM users WHERE role = 'student' ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(users.into_iter().map(UserResponse::from).collect())
    }

    /// Get a single student by ID
    pub async fn get_student(pool: &PgPool, id: Uuid) -> Result<UserResponse, AuthError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, first_name, last_name, role, is_active, created_at, updated_at, status
             FROM users WHERE id = $1 AND role = 'student'"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::UserNotFound)?;

        Ok(UserResponse::from(user))
    }

    /// Move student to waitlist
    pub async fn waitlist_student(pool: &PgPool, id: Uuid) -> Result<UserResponse, AuthError> {
        let user = sqlx::query_as::<_, User>(
            "UPDATE users SET status = 'waitlisted', updated_at = NOW()
             WHERE id = $1 AND role = 'student'
             RETURNING id, email, password_hash, first_name, last_name, role, is_active, created_at, updated_at, status"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::UserNotFound)?;

        Ok(UserResponse::from(user))
    }

    /// Accept student
    pub async fn accept_student(pool: &PgPool, id: Uuid) -> Result<UserResponse, AuthError> {
        let user = sqlx::query_as::<_, User>(
            "UPDATE users SET status = 'accepted', is_active = true, updated_at = NOW()
             WHERE id = $1 AND role = 'student'
             RETURNING id, email, password_hash, first_name, last_name, role, is_active, created_at, updated_at, status"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::UserNotFound)?;

        Ok(UserResponse::from(user))
    }

    /// Reject student
    pub async fn reject_student(pool: &PgPool, id: Uuid) -> Result<UserResponse, AuthError> {
        let user = sqlx::query_as::<_, User>(
            "UPDATE users SET status = 'rejected', is_active = false, updated_at = NOW()
             WHERE id = $1 AND role = 'student'
             RETURNING id, email, password_hash, first_name, last_name, role, is_active, created_at, updated_at, status"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::UserNotFound)?;

        Ok(UserResponse::from(user))
    }

    // ─── Mentor Management ───

    /// List all mentors
    pub async fn list_mentors(pool: &PgPool) -> Result<Vec<UserResponse>, AuthError> {
        let users = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, first_name, last_name, role, is_active, created_at, updated_at, status
             FROM users WHERE role = 'mentor' ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(users.into_iter().map(UserResponse::from).collect())
    }

    /// Get a single mentor by ID
    pub async fn get_mentor(pool: &PgPool, id: Uuid) -> Result<UserResponse, AuthError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, first_name, last_name, role, is_active, created_at, updated_at, status
             FROM users WHERE id = $1 AND role = 'mentor'"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::UserNotFound)?;

        Ok(UserResponse::from(user))
    }

    /// Update mentor details (first_name, last_name, email)
    pub async fn update_mentor(
        pool: &PgPool,
        id: Uuid,
        first_name: Option<String>,
        last_name: Option<String>,
        email: Option<String>,
    ) -> Result<UserResponse, AuthError> {
        // Build dynamic update
        let user = sqlx::query_as::<_, User>(
            "UPDATE users SET
                first_name = COALESCE($2, first_name),
                last_name = COALESCE($3, last_name),
                email = COALESCE($4, email),
                updated_at = NOW()
             WHERE id = $1 AND role = 'mentor'
             RETURNING id, email, password_hash, first_name, last_name, role, is_active, created_at, updated_at, status"
        )
        .bind(id)
        .bind(first_name)
        .bind(last_name)
        .bind(email)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::UserNotFound)?;

        Ok(UserResponse::from(user))
    }

    /// Delete a mentor
    pub async fn delete_mentor(pool: &PgPool, id: Uuid) -> Result<(), AuthError> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1 AND role = 'mentor'")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AuthError::UserNotFound);
        }
        Ok(())
    }

    /// Accept mentor
    pub async fn accept_mentor(pool: &PgPool, id: Uuid) -> Result<UserResponse, AuthError> {
        // Update user status to accepted
        let user = sqlx::query_as::<_, User>(
            "UPDATE users SET status = 'accepted', is_active = true, updated_at = NOW()
             WHERE id = $1 AND role = 'mentor'
             RETURNING id, email, password_hash, first_name, last_name, role, is_active, created_at, updated_at, status"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::UserNotFound)?;

        // Get all pending mentor applications for this user
        let applications = sqlx::query_scalar::<_, Uuid>(
            "SELECT course_id FROM mentor_applications WHERE user_id = $1 AND status = 'pending'",
        )
        .bind(id)
        .fetch_all(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        // Create course_enrollments for each application
        for course_id in applications {
            sqlx::query(
                "INSERT INTO course_enrollments (id, user_id, course_id, role, enrolled_at)
                 VALUES ($1, $2, $3, 'mentor', NOW())
                 ON CONFLICT (user_id, course_id) DO NOTHING"
            )
            .bind(Uuid::new_v4())
            .bind(id)
            .bind(course_id)
            .execute(pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

            // Update mentor_application status to accepted
            sqlx::query(
                "UPDATE mentor_applications SET status = 'accepted', updated_at = NOW()
                 WHERE user_id = $1 AND course_id = $2"
            )
            .bind(id)
            .bind(course_id)
            .execute(pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;
        }

        Ok(UserResponse::from(user))
    }

    /// Reject mentor
    pub async fn reject_mentor(pool: &PgPool, id: Uuid) -> Result<UserResponse, AuthError> {
        // Update user status to rejected
        let user = sqlx::query_as::<_, User>(
            "UPDATE users SET status = 'rejected', is_active = false, updated_at = NOW()
             WHERE id = $1 AND role = 'mentor'
             RETURNING id, email, password_hash, first_name, last_name, role, is_active, created_at, updated_at, status"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::UserNotFound)?;

        // Mark all pending mentor applications as rejected
        sqlx::query(
            "UPDATE mentor_applications SET status = 'rejected', updated_at = NOW()
             WHERE user_id = $1 AND status = 'pending'"
        )
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(UserResponse::from(user))
    }

    // ─── Dashboard ───

    /// Get dashboard statistics
    pub async fn get_dashboard_stats(pool: &PgPool) -> Result<serde_json::Value, AuthError> {
        let total_students = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE role = 'student'"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        let total_mentors = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE role = 'mentor'"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        let total_courses = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM courses"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        let pending_students = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE role = 'student' AND status IN ('pending', 'interview', 'waitlisted')"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        let pending_mentors = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE role = 'mentor' AND status = 'pending'"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(serde_json::json!({
            "total_students": total_students,
            "total_mentors": total_mentors,
            "total_courses": total_courses,
            "pending_approvals": {
                "students": pending_students,
                "mentors": pending_mentors
            }
        }))
    }
}
