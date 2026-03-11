use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::utils::AuthError;

/// Stub: get mentor dashboard data.
pub fn dashboard(user_id: Uuid, email: &str) -> Value {
    json!({
        "message": "Welcome to Mentor Dashboard",
        "user_id": user_id,
        "email": email,
        "role": "mentor",
        "permissions": [
            "manage_courses",
            "grade_assignments",
            "message_students",
            "view_student_progress",
            "create_assignments"
        ]
    })
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

/// Stub: get assigned students.
pub fn students(user_id: Uuid, email: &str) -> Value {
    json!({
        "mentor_id": user_id,
        "email": email,
        "students": [
            { "id": "student_1", "name": "Alice Johnson", "email": "alice@example.com", "status": "active" },
            { "id": "student_2", "name": "Bob Smith", "email": "bob@example.com", "status": "active" },
            { "id": "student_3", "name": "Carol White", "email": "carol@example.com", "status": "active" }
        ]
    })
}

/// Stub: get student progress.
pub fn student_progress(user_id: Uuid, student_id: &str) -> Value {
    json!({
        "mentor_id": user_id,
        "student_id": student_id,
        "progress": {
            "courses_enrolled": 2,
            "assignments_completed": 5,
            "assignments_pending": 2,
            "average_grade": "A-",
            "last_activity": chrono::Utc::now().to_rfc3339()
        }
    })
}

/// Stub: grade an assignment.
pub fn grade_assignment(user_id: Uuid, assignment_id: &str) -> Value {
    json!({
        "message": "Assignment graded successfully",
        "mentor_id": user_id,
        "assignment_id": assignment_id,
        "graded_at": chrono::Utc::now().to_rfc3339(),
        "status": "graded"
    })
}

/// Stub: create an assignment.
pub fn create_assignment(user_id: Uuid) -> Value {
    json!({
        "message": "Assignment created successfully",
        "mentor_id": user_id,
        "assignment_id": uuid::Uuid::new_v4().to_string(),
        "created_at": chrono::Utc::now().to_rfc3339(),
        "status": "active"
    })
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

/// Stub: get course assignments.
pub fn course_assignments(user_id: Uuid, course_id: &str) -> Value {
    json!({
        "mentor_id": user_id,
        "course_id": course_id,
        "assignments": [
            { "id": "assignment_1", "title": "Rust Basics", "due_date": "2024-02-15", "submissions": 3, "graded": 2 },
            { "id": "assignment_2", "title": "Web API Project", "due_date": "2024-02-22", "submissions": 2, "graded": 0 }
        ]
    })
}
