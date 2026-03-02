use axum::Router;
use sqlx::PgPool;

mod school_routes;
mod student_routes;

pub fn create_routes() -> Router<PgPool> {
    Router::new()
        .nest("/schools", school_routes::routes())
        .nest("/students", student_routes::routes())
}
