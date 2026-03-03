// OpenAPI documentation module
pub mod security;

use crate::docs::security::SecurityAddon;

#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "School Management API",
        version = "1.0.0",
        description = "REST API for school management system with role-based access control"
    ),
    servers(
        (url = "http://localhost:3000", description = "Development server")
    ),
    paths(
        // Auth endpoints
        crate::controllers::auth::register_admin,
        crate::controllers::auth::login_admin,
        crate::controllers::auth::register_student,
        crate::controllers::auth::login_student,
        crate::controllers::auth::register_mentor,
        crate::controllers::auth::login_mentor,
        crate::controllers::auth::refresh_token,
        crate::controllers::auth::logout,
        crate::controllers::auth::get_current_user,
        crate::controllers::auth::verify_token_endpoint,
        
        // Admin endpoints
        crate::controllers::admin::get_dashboard,
        crate::controllers::admin::get_all_users,
        crate::controllers::admin::get_statistics,
        crate::controllers::admin::deactivate_user,
        crate::controllers::admin::activate_user,
        
        // School endpoints
        crate::controllers::school::get_all_schools,
        crate::controllers::school::get_school_details,
        crate::controllers::school::create_school,
        crate::controllers::school::update_school,
        crate::controllers::school::delete_school,
        crate::controllers::school::get_school_statistics,
        
        // Student endpoints
        crate::controllers::student::get_dashboard,
        crate::controllers::student::get_profile,
        crate::controllers::student::get_courses,
        crate::controllers::student::submit_assignment,
        crate::controllers::student::get_grades,
        crate::controllers::student::message_mentor,
        
        // Mentor endpoints
        crate::controllers::mentor::get_dashboard,
        crate::controllers::mentor::get_profile,
        crate::controllers::mentor::get_students,
        crate::controllers::mentor::get_student_progress,
        crate::controllers::mentor::grade_assignment,
        crate::controllers::mentor::create_assignment,
        crate::controllers::mentor::message_student,
        crate::controllers::mentor::get_course_assignments
    ),
    components(
        schemas(
            // User models
            crate::models::User,
            crate::models::UserResponse,
            crate::models::UserRole,
            crate::models::RegisterRequest,
            crate::models::LoginRequest,
            crate::models::AuthResponse,
            crate::models::RefreshTokenRequest,
            crate::models::TokenResponse,
            
            // Error models
            crate::utils::ErrorResponse
        )
    ),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Authentication", description = "User authentication and authorization"),
        (name = "Admin", description = "Administrative operations (admin role required)"),
        (name = "Student", description = "Student operations (student role required)"),
        (name = "Mentor", description = "Mentor operations (mentor role required)"),
        (name = "School", description = "School management operations")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;
