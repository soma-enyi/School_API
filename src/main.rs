use axum::{Router, http::Method};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};
use tracing_subscriber;

mod models;
mod controllers;
mod services;
mod routes;
mod middlewares;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/school_db".to_string());

    // Create database connection pool
    tracing::info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    tracing::info!("Database connected successfully!");

    // Test the connection
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await?;

    tracing::info!("Database health check passed!");

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any)
        .allow_headers(Any);

    // Build application router
    let app = Router::new()
        .nest("/api", routes::create_routes())
        .layer(cors)
        .with_state(pool);

    // Start server on port 8080
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
