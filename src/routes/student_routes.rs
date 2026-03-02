use axum::{Router, routing::{get, post}};
use sqlx::PgPool;
use crate::controllers::student_controllers;

pub fn routes() -> Router<PgPool> {
    Router::new()
        .route("/", get(student_controllers::list_students).post(student_controllers::create_student))
        .route("/:id", get(student_controllers::get_student))
}
