use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service is healthy"),
    )
)]
pub async fn health_check() -> impl IntoResponse {
    let response = json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    (StatusCode::OK, Json(response))
}
