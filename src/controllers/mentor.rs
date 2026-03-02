use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::PgPool;

use crate::middlewares::MentorUser;
use crate::utils::AuthError;

pub struct MentorController;

impl MentorController {
    /// Get mentor dashboard
    /// GET /mentor/dashboard
    /// Requires: Mentor role
    pub async fn get_dashboard(
        mentor: MentorUser,
    ) -> Result<impl IntoResponse, AuthError> {
        let response = json!({
            "message": "Welcome to Mentor Dashboard",
            "user_id": mentor.user_id,
            "email": mentor.email,
            "role": "mentor",
            "permissions": [
                "manage_courses",
                "grade_assignments",
                "message_students",
                "view_student_progress",
                "create_assignments"
            ]
        });

        Ok((StatusCode::OK, Json(response)))
    }

    /// Get mentor profile
    /// GET /mentor/profile
    /// Requires: Mentor role
    pub async fn get_profile(
        mentor: MentorUser,
        State(pool): State<PgPool>,
    ) -> Result<impl IntoResponse, AuthError> {
        let user = sqlx::query_as::<_, (String, String, String, String, String)>(
            "SELECT id::text, email, first_name, last_name, role FROM users WHERE id = $1"
        )
        .bind(mentor.user_id.to_string())
        .fetch_optional(&pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::UserNotFound)?;

        let response = json!({
            "id": user.0,
            "email": user.1,
            "first_name": user.2,
            "last_name": user.3,
            "role": user.4
        });

        Ok((StatusCode::OK, Json(response)))
    }

    /// Get assigned students
    /// GET /mentor/students
    /// Requires: Mentor role
    pub async fn get_students(
        mentor: MentorUser,
    ) -> Result<impl IntoResponse, AuthError> {
        let response = json!({
            "mentor_id": mentor.user_id,
            "email": mentor.email,
            "students": [
                {
                    "id": "student_1",
                    "name": "Alice Johnson",
                    "email": "alice@example.com",
                    "status": "active"
                },
                {
                    "id": "student_2",
                    "name": "Bob Smith",
                    "email": "bob@example.com",
                    "status": "active"
                },
                {
                    "id": "student_3",
                    "name": "Carol White",
                    "email": "carol@example.com",
                    "status": "active"
                }
            ]
        });

        Ok((StatusCode::OK, Json(response)))
    }

    /// Get student progress
    /// GET /mentor/students/:student_id/progress
    /// Requires: Mentor role
    pub async fn get_student_progress(
        mentor: MentorUser,
        axum::extract::Path(student_id): axum::extract::Path<String>,
    ) -> Result<impl IntoResponse, AuthError> {
        let response = json!({
            "mentor_id": mentor.user_id,
            "student_id": student_id,
            "progress": {
                "courses_enrolled": 2,
                "assignments_completed": 5,
                "assignments_pending": 2,
                "average_grade": "A-",
                "last_activity": chrono::Utc::now().to_rfc3339()
            }
        });

        Ok((StatusCode::OK, Json(response)))
    }

    /// Grade assignment
    /// POST /mentor/assignments/:assignment_id/grade
    /// Requires: Mentor role
    pub async fn grade_assignment(
        mentor: MentorUser,
        axum::extract::Path(assignment_id): axum::extract::Path<String>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Result<impl IntoResponse, AuthError> {
        let response = json!({
            "message": "Assignment graded successfully",
            "mentor_id": mentor.user_id,
            "assignment_id": assignment_id,
            "graded_at": chrono::Utc::now().to_rfc3339(),
            "status": "graded"
        });

        Ok((StatusCode::OK, Json(response)))
    }

    /// Create assignment
    /// POST /mentor/assignments/create
    /// Requires: Mentor role
    pub async fn create_assignment(
        mentor: MentorUser,
        Json(_payload): Json<serde_json::Value>,
    ) -> Result<impl IntoResponse, AuthError> {
        let response = json!({
            "message": "Assignment created successfully",
            "mentor_id": mentor.user_id,
            "assignment_id": uuid::Uuid::new_v4().to_string(),
            "created_at": chrono::Utc::now().to_rfc3339(),
            "status": "active"
        });

        Ok((StatusCode::CREATED, Json(response)))
    }

    /// Message student
    /// POST /mentor/messages/student/:student_id
    /// Requires: Mentor role
    pub async fn message_student(
        mentor: MentorUser,
        axum::extract::Path(student_id): axum::extract::Path<String>,
        Json(_payload): Json<serde_json::Value>,
    ) -> Result<impl IntoResponse, AuthError> {
        let response = json!({
            "message": "Message sent to student successfully",
            "mentor_id": mentor.user_id,
            "student_id": student_id,
            "sent_at": chrono::Utc::now().to_rfc3339(),
            "status": "sent"
        });

        Ok((StatusCode::CREATED, Json(response)))
    }

    /// Get course assignments
    /// GET /mentor/courses/:course_id/assignments
    /// Requires: Mentor role
    pub async fn get_course_assignments(
        mentor: MentorUser,
        axum::extract::Path(course_id): axum::extract::Path<String>,
    ) -> Result<impl IntoResponse, AuthError> {
        let response = json!({
            "mentor_id": mentor.user_id,
            "course_id": course_id,
            "assignments": [
                {
                    "id": "assignment_1",
                    "title": "Rust Basics",
                    "due_date": "2024-02-15",
                    "submissions": 3,
                    "graded": 2
                },
                {
                    "id": "assignment_2",
                    "title": "Web API Project",
                    "due_date": "2024-02-22",
                    "submissions": 2,
                    "graded": 0
                }
            ]
        });

        Ok((StatusCode::OK, Json(response)))
    }
}
