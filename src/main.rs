mod controllers;
mod docs;
mod middlewares;
mod models;
mod routes;
mod services;
mod utils;

#[cfg(test)]
mod tests;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
    middleware,
    extract::Request,
};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tracing_subscriber;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use routes::auth_routes;
use crate::middlewares::auth_middleware;
use crate::docs::ApiDoc;
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
    let jwt_config_for_middleware = jwt_config.clone();

    // Build protected routes with state
    let protected_routes = Router::new()
        // Admin routes (protected)
        .route("/admin/dashboard", get(controllers::admin::get_dashboard))
        .route("/admin/users", get(controllers::admin::get_all_users))
        .route("/admin/statistics", get(controllers::admin::get_statistics))
        .route("/admin/users/:user_id/deactivate", axum::routing::post(controllers::admin::deactivate_user))
        .route("/admin/users/:user_id/activate", axum::routing::post(controllers::admin::activate_user))
        .route("/admin/schools", get(controllers::school::get_all_schools))
        .route("/admin/schools/create", axum::routing::post(controllers::school::create_school))
        .route("/admin/schools/:school_id", get(controllers::school::get_school_details))
        .route("/admin/schools/:school_id", axum::routing::put(controllers::school::update_school))
        .route("/admin/schools/:school_id", axum::routing::delete(controllers::school::delete_school))
        .route("/admin/schools/:school_id/statistics", get(controllers::school::get_school_statistics))
        
        // Student routes (protected)
        .route("/student/dashboard", get(controllers::student::get_dashboard))
        .route("/student/profile", get(controllers::student::get_profile))
        .route("/student/courses", get(controllers::student::get_courses))
        .route("/student/assignments/:assignment_id/submit", axum::routing::post(controllers::student::submit_assignment))
        .route("/student/grades", get(controllers::student::get_grades))
        .route("/student/messages/mentor", axum::routing::post(controllers::student::message_mentor))
        
        // Mentor routes (protected)
        .route("/mentor/dashboard", get(controllers::mentor::get_dashboard))
        .route("/mentor/profile", get(controllers::mentor::get_profile))
        .route("/mentor/students", get(controllers::mentor::get_students))
        .route("/mentor/students/:student_id/progress", get(controllers::mentor::get_student_progress))
        .route("/mentor/assignments/:assignment_id/grade", axum::routing::post(controllers::mentor::grade_assignment))
        .route("/mentor/assignments/create", axum::routing::post(controllers::mentor::create_assignment))
        .route("/mentor/messages/student/:student_id", axum::routing::post(controllers::mentor::message_student))
        .route("/mentor/courses/:course_id/assignments", get(controllers::mentor::get_course_assignments))
        
        // Apply authentication middleware to protected routes
        .layer(middleware::from_fn(move |req: Request, next| {
            let jwt_config = jwt_config_for_middleware.clone();
            async move {
                let mut req = req;
                req.extensions_mut().insert(jwt_config);
                auth_middleware(req, next).await
            }
        }))
        .with_state(pool.clone());

    // Build main router
    let app = Router::new()
        // Health check endpoint (public)
        .route("/health", get(health_check))
        
        // Authentication routes (public)
        .nest("/auth", auth_routes(pool.clone(), jwt_config.clone()))
        
        // Merge protected routes
        .merge(protected_routes)
        
        // Swagger UI
        .merge(
            SwaggerUi::new("/docs")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
        )
        
        // CORS layer
        .layer(CorsLayer::permissive());

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    tracing::info!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service is healthy"),
    )
)]
async fn health_check() -> impl IntoResponse {
    let response = json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    (StatusCode::OK, Json(response))
}
