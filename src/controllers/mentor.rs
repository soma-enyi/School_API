use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::utils::AuthError;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAssignmentPayload {
    #[schema(format = "uuid")]
    pub course_id: Uuid,
    #[schema(example = "Rust Ownership Deep Dive")]
    pub title: String,
    #[schema(example = "Implement ownership and borrowing exercises")]
    pub description: Option<String>,
    #[schema(format = "date-time")]
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct GradeAssignmentPayload {
    #[schema(format = "uuid")]
    pub student_id: Uuid,
    #[schema(example = 87, minimum = 0, maximum = 100)]
    pub grade_score: i32,
    #[schema(example = "Good work. Please improve edge-case handling.")]
    pub feedback: Option<String>,
}

/// Get mentor dashboard data from DB.
pub async fn dashboard(
    pool: &PgPool,
    user_id: Uuid,
    email: &str,
) -> Result<Value, AuthError> {
    let mentor = sqlx::query_as::<_, (String, String, String, String, String)>(
        "SELECT first_name, last_name, email, role, status FROM users WHERE id = $1 AND role = 'mentor'",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?
    .ok_or(AuthError::UserNotFound)?;

    let courses_assigned = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM course_enrollments WHERE user_id = $1 AND role = 'mentor'",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    let students_assigned = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(DISTINCT students.user_id)
         FROM course_enrollments mentors
         JOIN course_enrollments students ON students.course_id = mentors.course_id
         WHERE mentors.user_id = $1
           AND mentors.role = 'mentor'
           AND students.role = 'student'",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    let assignments_created = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM assignments WHERE created_by = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    let submissions_received = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)
         FROM assignment_submissions s
         JOIN assignments a ON a.id = s.assignment_id
         WHERE a.created_by = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    let due_soon = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)
         FROM assignments
         WHERE created_by = $1
           AND due_date IS NOT NULL
           AND due_date >= NOW()
           AND due_date <= NOW() + INTERVAL '7 days'",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    let recent_assignments = sqlx::query_as::<_, (Uuid, String, Uuid, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>, i64)>(
        "SELECT
            a.id,
            a.title,
            a.course_id,
            a.created_at,
            a.due_date,
            COUNT(s.id)::bigint AS submissions
         FROM assignments a
         LEFT JOIN assignment_submissions s ON s.assignment_id = a.id
         WHERE a.created_by = $1
         GROUP BY a.id, a.title, a.course_id, a.created_at, a.due_date
         ORDER BY a.created_at DESC
         LIMIT 5",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    let recent_assignments = recent_assignments
        .into_iter()
        .map(|a| {
            json!({
                "id": a.0,
                "title": a.1,
                "course_id": a.2,
                "created_at": a.3,
                "due_date": a.4,
                "submissions": a.5
            })
        })
        .collect::<Vec<_>>();

    Ok(json!({
        "mentor": {
            "id": user_id,
            "first_name": mentor.0,
            "last_name": mentor.1,
            "email": mentor.2,
            "role": mentor.3,
            "status": mentor.4
        },
        "summary": {
            "courses_assigned": courses_assigned,
            "students_assigned": students_assigned,
            "assignments_created": assignments_created,
            "submissions_received": submissions_received,
            "assignments_due_in_7_days": due_soon
        },
        "recent_assignments": recent_assignments,
        "permissions": [
            "manage_courses",
            "grade_assignments",
            "message_students",
            "view_student_progress",
            "create_assignments"
        ],
        "context_email": email
    }))
}

/// Stub: get mentor profile from DB.
pub async fn profile(pool: &PgPool, user_id: Uuid) -> Result<Value, AuthError> {
    let user = sqlx::query_as::<_, (String, String, String, String, String)>(
        "SELECT id::text, email, first_name, last_name, role FROM users WHERE id = $1",
    )
    .bind(user_id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?
    .ok_or(AuthError::UserNotFound)?;

    Ok(json!({
        "id": user.0,
        "email": user.1,
        "first_name": user.2,
        "last_name": user.3,
        "role": user.4
    }))
}

/// Get assigned students for a mentor from DB.
pub async fn students(
    pool: &PgPool,
    user_id: Uuid,
    email: &str,
) -> Result<Value, AuthError> {
    let students = sqlx::query_as::<_, (Uuid, String, String, String, String, i64)>(
        "SELECT
            u.id,
            u.first_name,
            u.last_name,
            u.email,
            u.status,
            COUNT(DISTINCT s.course_id)::bigint AS course_count
         FROM course_enrollments m
         JOIN course_enrollments s
           ON s.course_id = m.course_id
          AND s.role = 'student'
         JOIN users u ON u.id = s.user_id
         WHERE m.user_id = $1
           AND m.role = 'mentor'
         GROUP BY u.id, u.first_name, u.last_name, u.email, u.status
         ORDER BY u.last_name, u.first_name",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    let students = students
        .into_iter()
        .map(|s| {
            json!({
                "id": s.0,
                "first_name": s.1,
                "last_name": s.2,
                "email": s.3,
                "status": s.4,
                "courses_enrolled_with_mentor": s.5
            })
        })
        .collect::<Vec<_>>();

    Ok(json!({
        "mentor_id": user_id,
        "email": email,
        "total_students": students.len(),
        "students": students
    }))
}

/// Get a specific student's progress under this mentor from DB.
pub async fn student_progress(
    pool: &PgPool,
    user_id: Uuid,
    student_id: &str,
) -> Result<Value, AuthError> {
    let is_assigned = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1
            FROM course_enrollments m
            JOIN course_enrollments s ON s.course_id = m.course_id
            WHERE m.user_id = $1
              AND m.role = 'mentor'
              AND s.user_id::text = $2
              AND s.role = 'student'
        )",
    )
    .bind(user_id)
    .bind(student_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    if !is_assigned {
        return Err(AuthError::Forbidden);
    }

    let student = sqlx::query_as::<_, (Uuid, String, String, String, String)>(
        "SELECT id, first_name, last_name, email, status
         FROM users
         WHERE id::text = $1
           AND role = 'student'",
    )
    .bind(student_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?
    .ok_or(AuthError::UserNotFound)?;

    let courses_shared = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(DISTINCT s.course_id)::bigint
         FROM course_enrollments m
         JOIN course_enrollments s ON s.course_id = m.course_id
         WHERE m.user_id = $1
           AND m.role = 'mentor'
           AND s.user_id::text = $2
           AND s.role = 'student'",
    )
    .bind(user_id)
    .bind(student_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    let assignments_total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::bigint
         FROM assignments a
         JOIN course_enrollments m ON m.course_id = a.course_id
         JOIN course_enrollments s ON s.course_id = a.course_id
         WHERE m.user_id = $1
           AND m.role = 'mentor'
           AND s.user_id::text = $2
           AND s.role = 'student'",
    )
    .bind(user_id)
    .bind(student_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    let assignments_submitted = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::bigint
         FROM assignment_submissions sub
         JOIN assignments a ON a.id = sub.assignment_id
         JOIN course_enrollments m ON m.course_id = a.course_id
         WHERE m.user_id = $1
           AND m.role = 'mentor'
           AND sub.student_id::text = $2",
    )
    .bind(user_id)
    .bind(student_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    let recent_submissions = sqlx::query_as::<_, (Uuid, String, Uuid, chrono::DateTime<chrono::Utc>)>(
        "SELECT
            sub.assignment_id,
            a.title,
            a.course_id,
            sub.submitted_at
         FROM assignment_submissions sub
         JOIN assignments a ON a.id = sub.assignment_id
         JOIN course_enrollments m ON m.course_id = a.course_id
         WHERE m.user_id = $1
           AND m.role = 'mentor'
           AND sub.student_id::text = $2
         ORDER BY sub.submitted_at DESC
         LIMIT 5",
    )
    .bind(user_id)
    .bind(student_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    let recent_submissions = recent_submissions
        .into_iter()
        .map(|r| {
            json!({
                "assignment_id": r.0,
                "assignment_title": r.1,
                "course_id": r.2,
                "submitted_at": r.3
            })
        })
        .collect::<Vec<_>>();

    Ok(json!({
        "mentor_id": user_id,
        "student": {
            "id": student.0,
            "first_name": student.1,
            "last_name": student.2,
            "email": student.3,
            "status": student.4
        },
        "progress": {
            "courses_shared": courses_shared,
            "assignments_total": assignments_total,
            "assignments_submitted": assignments_submitted,
            "assignments_pending": (assignments_total - assignments_submitted).max(0)
        },
        "recent_submissions": recent_submissions
    }))
}

/// Grade a student's assignment submission.
pub async fn grade_assignment(
    pool: &PgPool,
    user_id: Uuid,
    assignment_id: &str,
    payload: GradeAssignmentPayload,
) -> Result<Value, AuthError> {
    if !(0..=100).contains(&payload.grade_score) {
        return Err(AuthError::Forbidden);
    }

    let assignment_course = sqlx::query_scalar::<_, Uuid>(
        "SELECT course_id FROM assignments WHERE id::text = $1",
    )
    .bind(assignment_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?
    .ok_or(AuthError::UserNotFound)?;

    let can_grade = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1
            FROM course_enrollments
            WHERE user_id = $1
              AND role = 'mentor'
              AND course_id = $2
        )",
    )
    .bind(user_id)
    .bind(assignment_course)
    .fetch_one(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    if !can_grade {
        return Err(AuthError::Forbidden);
    }

    let graded = sqlx::query_as::<_, (Uuid, Uuid, Uuid, i32, Option<String>, Uuid, DateTime<Utc>)>(
        "UPDATE assignment_submissions
         SET grade_score = $3,
             feedback = $4,
             graded_by = $5,
             graded_at = NOW()
         WHERE assignment_id::text = $1
           AND student_id = $2
         RETURNING id, assignment_id, student_id, grade_score, feedback, graded_by, graded_at",
    )
    .bind(assignment_id)
    .bind(payload.student_id)
    .bind(payload.grade_score)
    .bind(payload.feedback)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?
    .ok_or(AuthError::UserNotFound)?;

    Ok(json!({
        "message": "Assignment graded successfully",
        "mentor_id": user_id,
        "submission": {
            "id": graded.0,
            "assignment_id": graded.1,
            "student_id": graded.2,
            "grade_score": graded.3,
            "feedback": graded.4,
            "graded_by": graded.5,
            "graded_at": graded.6
        }
    }))
}

/// Create a new assignment for a mentor's assigned course.
pub async fn create_assignment(
    pool: &PgPool,
    user_id: Uuid,
    payload: CreateAssignmentPayload,
) -> Result<Value, AuthError> {
    let can_create = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1
            FROM course_enrollments
            WHERE user_id = $1
              AND role = 'mentor'
              AND course_id = $2
        )",
    )
    .bind(user_id)
    .bind(payload.course_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    if !can_create {
        return Err(AuthError::Forbidden);
    }

    let assignment = sqlx::query_as::<_, (Uuid, Uuid, String, Option<String>, Option<DateTime<Utc>>, Uuid, DateTime<Utc>)>(
        "INSERT INTO assignments (course_id, title, description, due_date, created_by)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, course_id, title, description, due_date, created_by, created_at",
    )
    .bind(payload.course_id)
    .bind(payload.title)
    .bind(payload.description)
    .bind(payload.due_date)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    Ok(json!({
        "message": "Assignment created successfully",
        "mentor_id": user_id,
        "assignment": {
            "id": assignment.0,
            "course_id": assignment.1,
            "title": assignment.2,
            "description": assignment.3,
            "due_date": assignment.4,
            "created_by": assignment.5,
            "created_at": assignment.6
        }
    }))
}

/// Stub: message a student.
pub fn message_student(user_id: Uuid, student_id: &str) -> Value {
    json!({
        "message": "Message sent to student successfully",
        "mentor_id": user_id,
        "student_id": student_id,
        "sent_at": chrono::Utc::now().to_rfc3339(),
        "status": "sent"
    })
}

/// Get assignments for a mentor's course from DB.
pub async fn course_assignments(
    pool: &PgPool,
    user_id: Uuid,
    course_id: &str,
) -> Result<Value, AuthError> {
    let is_assigned = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1
            FROM course_enrollments
            WHERE user_id = $1
              AND role = 'mentor'
              AND course_id::text = $2
        )",
    )
    .bind(user_id)
    .bind(course_id)
    .fetch_one(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    if !is_assigned {
        return Err(AuthError::Forbidden);
    }

    let course = sqlx::query_as::<_, (Uuid, String, Option<String>)>(
        "SELECT id, name, description
         FROM courses
         WHERE id::text = $1",
    )
    .bind(course_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?
    .ok_or(AuthError::UserNotFound)?;

    let assignments = sqlx::query_as::<_, (Uuid, String, Option<String>, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>, Uuid, i64)>(
        "SELECT
            a.id,
            a.title,
            a.description,
            a.created_at,
            a.due_date,
            a.created_by,
            COUNT(sub.id)::bigint AS submissions
         FROM assignments a
         LEFT JOIN assignment_submissions sub ON sub.assignment_id = a.id
         WHERE a.course_id::text = $1
         GROUP BY a.id, a.title, a.description, a.created_at, a.due_date, a.created_by
         ORDER BY a.created_at DESC",
    )
    .bind(course_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    let assignments = assignments
        .into_iter()
        .map(|a| {
            json!({
                "id": a.0,
                "title": a.1,
                "description": a.2,
                "created_at": a.3,
                "due_date": a.4,
                "created_by": a.5,
                "created_by_me": a.5 == user_id,
                "submissions": a.6
            })
        })
        .collect::<Vec<_>>();

    Ok(json!({
        "mentor_id": user_id,
        "course": {
            "id": course.0,
            "name": course.1,
            "description": course.2
        },
        "total_assignments": assignments.len(),
        "assignments": assignments
    }))
}
