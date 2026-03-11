use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::utils::AuthError;

/// Stub: get student dashboard data.
pub fn dashboard(user_id: Uuid, email: &str) -> Value {
    json!({
        "message": "Welcome to Student Dashboard",
        "user_id": user_id,
        "email": email,
        "role": "student",
        "permissions": [
            "view_courses",
            "submit_assignments",
            "view_grades",
            "message_mentor",
            "view_schedule"
        ]
    })
}

/// Stub: get student profile from DB.
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

/// Stub: list enrolled courses.
pub fn courses(user_id: Uuid, email: &str) -> Value {
    json!({
        "student_id": user_id,
        "email": email,
        "courses": [
            { "id": "course_1", "name": "Introduction to Rust", "mentor": "John Doe", "status": "active" },
            { "id": "course_2", "name": "Web Development with Axum", "mentor": "Jane Smith", "status": "active" }
        ]
    })
}

/// Stub: submit an assignment.
pub fn submit_assignment(user_id: Uuid, assignment_id: &str) -> Value {
    json!({
        "message": "Assignment submitted successfully",
        "student_id": user_id,
        "assignment_id": assignment_id,
        "submitted_at": chrono::Utc::now().to_rfc3339(),
        "status": "pending_review"
    })
}

/// Stub: get grades.
pub fn grades(user_id: Uuid, email: &str) -> Value {
    json!({
        "student_id": user_id,
        "email": email,
        "grades": [
            { "course": "Introduction to Rust", "assignment": "Assignment 1", "grade": "A", "score": 95 },
            { "course": "Web Development with Axum", "assignment": "Project 1", "grade": "B+", "score": 87 }
        ]
    })
}

/// Stub: message mentor.
pub fn message_mentor(user_id: Uuid) -> Value {
    json!({
        "message": "Message sent to mentor successfully",
        "student_id": user_id,
        "sent_at": chrono::Utc::now().to_rfc3339(),
        "status": "sent"
    })
}
