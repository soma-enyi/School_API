use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;

use crate::controllers::{
    register_admin, login_admin, register_student, login_student,
    register_mentor, login_mentor, refresh_token, logout,
    get_current_user, verify_token_endpoint
};
use crate::controllers::auth_controllers::AuthController;
use crate::utils::JwtConfig;

pub fn auth_routes(pool: PgPool, jwt_config: JwtConfig) -> Router {
    Router::new()
        // Admin authentication routes
        .route("/admin/register", post(register_admin))
        .route("/admin/login", post(login_admin))
        
        // Student authentication routes
        .route("/student/register", post(register_student))
        .route("/student/login", post(login_student))
        
        // Mentor authentication routes
        .route("/mentor/register", post(register_mentor))
        .route("/mentor/login", post(login_mentor))
        
        // OTP routes
        .route("/verify-otp", post(AuthController::verify_otp_login))
        .route("/resend-otp", post(AuthController::resend_otp))
        
        // Common authentication routes
        .route("/refresh", post(refresh_token))
        .route("/logout", post(logout))
        .route("/me", get(get_current_user))
        .route("/verify", post(verify_token_endpoint))
        
        .with_state((pool, jwt_config))
}
