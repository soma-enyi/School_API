use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;

use crate::controllers::{
    // Admin
    get_dashboard as admin_get_dashboard,
    get_all_users,
    get_statistics,
    deactivate_user,
    activate_user,
    // School
    get_all_schools,
    create_school,
    get_school_details,
    update_school,
    delete_school,
    get_school_statistics,
    // Student
    get_dashboard as student_get_dashboard,
    get_profile as student_get_profile,
    get_courses,
    submit_assignment,
    get_grades,
    message_mentor,
    // Mentor
    get_dashboard as mentor_get_dashboard,
    get_profile as mentor_get_profile,
    get_students,
    get_student_progress,
    grade_assignment,
    create_assignment,
    message_student,
    get_course_assignments,
};
use crate::middlewares::auth_middleware;
use crate::utils::JwtConfig;

/// Protected routes that require authentication
pub fn protected_routes(pool: PgPool, jwt_config: JwtConfig) -> Router<(PgPool, JwtConfig)> {
    Router::new()
        // Admin routes
        .route("/admin/dashboard", get(admin_get_dashboard))
        .route("/admin/users", get(get_all_users))
        .route("/admin/statistics", get(get_statistics))
        .route("/admin/users/:user_id/deactivate", post(deactivate_user))
        .route("/admin/users/:user_id/activate", post(activate_user))
        
        // School management routes (admin only)
        .route("/admin/schools", get(get_all_schools))
        .route("/admin/schools/create", post(create_school))
        .route("/admin/schools/:school_id", get(get_school_details))
        .route("/admin/schools/:school_id", put(update_school))
        .route("/admin/schools/:school_id", delete(delete_school))
        .route("/admin/schools/:school_id/statistics", get(get_school_statistics))
        
        // Student routes
        .route("/student/dashboard", get(student_get_dashboard))
        .route("/student/profile", get(student_get_profile))
        .route("/student/courses", get(get_courses))
        .route("/student/assignments/:assignment_id/submit", post(submit_assignment))
        .route("/student/grades", get(get_grades))
        .route("/student/messages/mentor", post(message_mentor))
        
        // Mentor routes
        .route("/mentor/dashboard", get(mentor_get_dashboard))
        .route("/mentor/profile", get(mentor_get_profile))
        .route("/mentor/students", get(get_students))
        .route("/mentor/students/:student_id/progress", get(get_student_progress))
        .route("/mentor/assignments/:assignment_id/grade", post(grade_assignment))
        .route("/mentor/assignments/create", post(create_assignment))
        .route("/mentor/messages/student/:student_id", post(message_student))
        .route("/mentor/courses/:course_id/assignments", get(get_course_assignments))
        
        .with_state((pool, jwt_config))
}
