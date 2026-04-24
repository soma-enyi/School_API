// OpenAPI documentation module
pub mod security;

use crate::docs::security::SecurityAddon;

#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "Course Flow API",
        version = "1.0.0",
        description = "REST API for course flow management system with role-based access control"
    ),
    servers(
        (url = "http://localhost:3000", description = "Development server")
    ),
    paths(
        // Health
        crate::routes::health::health_check,

        // Auth endpoints
        crate::routes::auth_routes::register_admin,
        crate::routes::auth_routes::login_admin,
        crate::routes::auth_routes::register_student,
        crate::routes::auth_routes::login_student,
        crate::routes::auth_routes::register_mentor,
        crate::routes::auth_routes::login_mentor,
        crate::routes::auth_routes::refresh_token,
        crate::routes::auth_routes::logout,
        crate::routes::auth_routes::get_current_user,
        crate::routes::auth_routes::verify_token_endpoint,
        
        // Admin endpoints
        crate::routes::admin_routes::admin_dashboard,
        crate::routes::admin_routes::list_students,
        crate::routes::admin_routes::get_student,
        crate::routes::admin_routes::waitlist_student,
        crate::routes::admin_routes::accept_student,
        crate::routes::admin_routes::reject_student,
        crate::routes::admin_routes::list_mentors,
        crate::routes::admin_routes::get_mentor,
        crate::routes::admin_routes::update_mentor,
        crate::routes::admin_routes::delete_mentor,
        crate::routes::admin_routes::accept_mentor,
        crate::routes::admin_routes::reject_mentor,
        crate::routes::admin_routes::create_course,
        crate::routes::admin_routes::list_courses,
        crate::routes::admin_routes::update_course,
        crate::routes::admin_routes::delete_course,
        crate::routes::admin_routes::process_dropout_alerts,
        
        // Application review (admin)
        crate::routes::admin_routes::list_applications,
        crate::routes::admin_routes::get_application,
        crate::routes::admin_routes::accept_application,
        crate::routes::admin_routes::waitlist_application,
        crate::routes::admin_routes::enroll_application,
        crate::routes::admin_routes::reject_application,
        
        // Public application
        crate::routes::application_routes::submit_application,
        crate::routes::application_routes::list_available_courses,
        
        // Newsletter endpoints
        crate::routes::newsletter_routes::subscribe,
        crate::routes::newsletter_routes::unsubscribe,
        crate::routes::newsletter_routes::send_newsletter,
        
        // Student endpoints
        crate::routes::student_routes::get_dashboard,
        crate::routes::student_routes::get_profile,
        crate::routes::student_routes::get_courses,
        crate::routes::student_routes::submit_assignment,
        crate::routes::student_routes::get_grades,
        crate::routes::student_routes::message_mentor,
        
        // Mentor endpoints
        crate::routes::mentor_routes::get_dashboard,
        crate::routes::mentor_routes::get_profile,
        crate::routes::mentor_routes::get_students,
        crate::routes::mentor_routes::get_student_progress,
        crate::routes::mentor_routes::grade_assignment,
        crate::routes::mentor_routes::create_assignment,
        crate::routes::mentor_routes::message_student,
        crate::routes::mentor_routes::get_course_assignments,

        // Attendance endpoints
        crate::routes::attendance_routes::create_session,
        crate::routes::attendance_routes::list_sessions,
        crate::routes::attendance_routes::generate_qr,
        crate::routes::attendance_routes::mark,
        crate::routes::attendance_routes::get_by_date,
        crate::routes::attendance_routes::get_summary,
        crate::routes::attendance_routes::session_report,
        crate::routes::attendance_routes::checkin_pin,
        crate::routes::attendance_routes::checkin_qr,
        crate::routes::attendance_routes::checkin_login,
        crate::routes::attendance_routes::get_by_student,
        crate::routes::attendance_routes::set_pin
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
            
            // Course models
            crate::models::Course,
            crate::models::CourseResponse,
            crate::models::CreateCourseRequest,
            crate::models::UpdateCourseRequest,
            
            // Application models
            crate::models::ApplicationRequest,
            crate::models::ApplicationResponse,
            
            // Error models
            crate::utils::ErrorResponse,
            
            // Newsletter models
            crate::services::newsletter_service::SubscribeRequest,
            crate::services::newsletter_service::UnsubscribeRequest,
            crate::services::newsletter_service::SendNewsletterRequest,
            crate::services::newsletter_service::SubscriberResponse,

            // Attendance models
            crate::models::attendance::Attendance,
            crate::models::attendance::MarkAttendanceRequest,
            crate::models::attendance::ClassSession,
            crate::models::attendance::CreateSessionRequest,
            crate::models::attendance::QrCode,
            crate::models::attendance::PinCheckInRequest,
            crate::models::attendance::QrCheckInRequest,
            crate::models::attendance::LoginCheckInRequest,
            crate::models::attendance::SetPinRequest,
            crate::models::attendance::AttendanceSummary,
            crate::models::attendance::SessionAttendanceReport
        )
    ),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Applications", description = "Public course application endpoints"),
        (name = "Authentication", description = "User authentication and authorization"),
        (name = "Admin", description = "Administrative operations (admin role required)"),
        (name = "Student", description = "Student operations (student role required)"),
        (name = "Mentor", description = "Mentor operations (mentor role required)"),
        (name = "Newsletter", description = "Newsletter subscribe/unsubscribe and send"),
        (name = "Attendance", description = "Class sessions, self-check-in (PIN/QR/login), and attendance reports")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;
