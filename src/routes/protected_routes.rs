use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;

use crate::controllers::{AdminController, StudentController, MentorController, SchoolController};
use crate::middlewares::auth_middleware;
use crate::utils::JwtConfig;

/// Protected routes that require authentication
pub fn protected_routes(pool: PgPool, jwt_config: JwtConfig) -> Router<(PgPool, JwtConfig)> {
    Router::new()
        // Admin routes
        .route("/admin/dashboard", get(AdminController::get_dashboard))
        .route("/admin/users", get(AdminController::get_all_users))
        .route("/admin/statistics", get(AdminController::get_statistics))
        .route("/admin/users/:user_id/deactivate", post(AdminController::deactivate_user))
        .route("/admin/users/:user_id/activate", post(AdminController::activate_user))
        
        // School management routes (admin only)
        .route("/admin/schools", get(SchoolController::get_all_schools))
        .route("/admin/schools/create", post(SchoolController::create_school))
        .route("/admin/schools/:school_id", get(SchoolController::get_school_details))
        .route("/admin/schools/:school_id", put(SchoolController::update_school))
        .route("/admin/schools/:school_id", delete(SchoolController::delete_school))
        .route("/admin/schools/:school_id/statistics", get(SchoolController::get_school_statistics))
        
        // Student routes
        .route("/student/dashboard", get(StudentController::get_dashboard))
        .route("/student/profile", get(StudentController::get_profile))
        .route("/student/courses", get(StudentController::get_courses))
        .route("/student/assignments/:assignment_id/submit", post(StudentController::submit_assignment))
        .route("/student/grades", get(StudentController::get_grades))
        .route("/student/messages/mentor", post(StudentController::message_mentor))
        
        // Mentor routes
        .route("/mentor/dashboard", get(MentorController::get_dashboard))
        .route("/mentor/profile", get(MentorController::get_profile))
        .route("/mentor/students", get(MentorController::get_students))
        .route("/mentor/students/:student_id/progress", get(MentorController::get_student_progress))
        .route("/mentor/assignments/:assignment_id/grade", post(MentorController::grade_assignment))
        .route("/mentor/assignments/create", post(MentorController::create_assignment))
        .route("/mentor/messages/student/:student_id", post(MentorController::message_student))
        .route("/mentor/courses/:course_id/assignments", get(MentorController::get_course_assignments))
        
        .with_state((pool, jwt_config))
}
