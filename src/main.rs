mod controllers;
mod middlewares;
mod models;
mod routes;
mod services;
mod utils;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
    middleware,
};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

use routes::auth_routes;
use crate::controllers::{AdminController, StudentController, MentorController, SchoolController};
use crate::middlewares::auth_middleware;
use utils::JwtConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:password@localhost/school_db".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    tracing::info!("Database migrations completed successfully");

    // JWT configuration
    let jwt_config = JwtConfig::from_env();

    // Build router
    let protected_routes = Router::new()
        // Admin routes (protected)
        .route("/admin/dashboard", get(AdminController::get_dashboard))
        .route("/admin/users", get(AdminController::get_all_users))
        .route("/admin/statistics", get(AdminController::get_statistics))
        .route("/admin/users/:user_id/deactivate", axum::routing::post(AdminController::deactivate_user))
        .route("/admin/users/:user_id/activate", axum::routing::post(AdminController::activate_user))
        .route("/admin/schools", get(SchoolController::get_all_schools))
        .route("/admin/schools/create", axum::routing::post(SchoolController::create_school))
        .route("/admin/schools/:school_id", get(SchoolController::get_school_details))
        .route("/admin/schools/:school_id", axum::routing::put(SchoolController::update_school))
        .route("/admin/schools/:school_id", axum::routing::delete(SchoolController::delete_school))
        .route("/admin/schools/:school_id/statistics", get(SchoolController::get_school_statistics))
        
        // Student routes (protected)
        .route("/student/dashboard", get(StudentController::get_dashboard))
        .route("/student/profile", get(StudentController::get_profile))
        .route("/student/courses", get(StudentController::get_courses))
        .route("/student/assignments/:assignment_id/submit", axum::routing::post(StudentController::submit_assignment))
        .route("/student/grades", get(StudentController::get_grades))
        .route("/student/messages/mentor", axum::routing::post(StudentController::message_mentor))
        
        // Mentor routes (protected)
        .route("/mentor/dashboard", get(MentorController::get_dashboard))
        .route("/mentor/profile", get(MentorController::get_profile))
        .route("/mentor/students", get(MentorController::get_students))
        .route("/mentor/students/:student_id/progress", get(MentorController::get_student_progress))
        .route("/mentor/assignments/:assignment_id/grade", axum::routing::post(MentorController::grade_assignment))
        .route("/mentor/assignments/create", axum::routing::post(MentorController::create_assignment))
        .route("/mentor/messages/student/:student_id", axum::routing::post(MentorController::message_student))
        .route("/mentor/courses/:course_id/assignments", get(MentorController::get_course_assignments))
        
        // Apply authentication middleware only to protected routes
        .layer(middleware::from_fn(move |req, next| {
            let jwt_config = jwt_config.clone();
            async move {
                let mut req = req;
                req.extensions_mut().insert(jwt_config);
                auth_middleware(req, next).await
            }
        }))
        .with_state((pool.clone(), jwt_config.clone()));

    let app = Router::new()
        // Health check endpoint (public)
        .route("/health", get(health_check))
        
        // Authentication routes (public)
        .nest("/auth", auth_routes(pool.clone(), jwt_config.clone()))
        
        // Protected routes
        .merge(protected_routes)
        
        // CORS layer
        .layer(CorsLayer::permissive())
        
        .with_state((pool, jwt_config));

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    tracing::info!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    let response = json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    (StatusCode::OK, Json(response))
}
