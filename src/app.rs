use axum::{
    extract::Request,
    middleware,
    response::Json,
    routing::get,
    Router,
};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::docs::ApiDoc;
use crate::middlewares::auth_middleware;
use crate::routes::admin_routes::admin_routes;
use crate::routes::auth_routes::auth_routes;
use crate::routes::application_routes::application_routes;
use crate::routes::attendance_routes::attendance_routes;
use crate::routes::student_routes::student_routes;
use crate::routes::mentor_routes::mentor_routes;
use crate::routes::newsletter_routes::newsletter_routes;
use crate::state::AppState;
use crate::utils::JwtConfig;

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

/// Build the full application Router with all routes, middleware, and state.
pub fn build_app(pool: PgPool) -> Router {
    // Create application state (includes email service)
    let app_state = AppState::new(pool.clone());

    // JWT configuration
    let jwt_config = JwtConfig::from_env();
    let jwt_config_for_admin = jwt_config.clone();
    let jwt_config_for_pool_routes = jwt_config.clone();

    // Admin routes — protected by auth middleware, uses AppState (for email)
    let admin_protected = Router::new()
        .nest("/admin", admin_routes())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api-docs/openapi.json", get(openapi_json))
        .layer(middleware::from_fn(move |req: Request, next| {
            let jwt_config = jwt_config_for_admin.clone();
            async move {
                let mut req = req;
                req.extensions_mut().insert(jwt_config);
                auth_middleware(req, next).await
            }
        }))
        .with_state(app_state.clone());

    // Student & mentor routes — protected by auth middleware, uses PgPool
    let student_mentor_protected = Router::new()
        .merge(student_routes())
        .merge(mentor_routes())
        .merge(attendance_routes())
        .layer(middleware::from_fn(move |req: Request, next| {
            let jwt_config = jwt_config_for_pool_routes.clone();
            async move {
                let mut req = req;
                req.extensions_mut().insert(jwt_config);
                auth_middleware(req, next).await
            }
        }))
        .with_state(pool.clone());

    // Assemble the final router
    Router::new()
        // Health check (public)
        .route("/health", get(crate::routes::health::health_check))
        // Public application endpoints (no auth)
        .merge(application_routes(pool.clone()))
        // Newsletter routes (public subscribe/unsubscribe, admin send)
        .merge(newsletter_routes(app_state.clone()))
        // Auth routes (public)
        .nest("/auth", auth_routes(pool.clone(), jwt_config.clone()))
        // Protected routes
        .merge(admin_protected)
        .merge(student_mentor_protected)
        // CORS
        .layer(CorsLayer::permissive())
}