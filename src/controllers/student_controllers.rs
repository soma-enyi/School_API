use axum::{extract::{State, Path}, Json, http::StatusCode};
use sqlx::PgPool;
use crate::models::user::{Student, CreateStudent};
use crate::services::student_services;

pub async fn list_students(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Student>>, StatusCode> {
    student_services::get_all_students(&pool)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_student(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<Json<Student>, StatusCode> {
    student_services::get_student_by_id(&pool, id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

pub async fn create_student(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateStudent>,
) -> Result<Json<Student>, StatusCode> {
    student_services::create_student(&pool, payload)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
