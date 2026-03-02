use axum::{Router, routing::{get, post}};
use sqlx::PgPool;
use crate::controllers::school_controllers;

pub fn routes() -> Router<PgPool> {
    Router::new()
        .route("/", get(school_controllers::list_schools).post(school_controllers::create_school))
        .route("/:id", get(school_controllers::get_school))
}
