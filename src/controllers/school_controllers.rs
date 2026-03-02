use axum::{extract::{State, Path}, Json, http::StatusCode};
use sqlx::PgPool;
use crate::models::school::{School, CreateSchool};
use crate::services::school_services;

pub async fn list_schools(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<School>>, StatusCode> {
    school_services::get_all_schools(&pool)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_school(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<Json<School>, StatusCode> {
    school_services::get_school_by_id(&pool, id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

pub async fn create_school(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateSchool>,
) -> Result<Json<School>, StatusCode> {
    school_services::create_school(&pool, payload)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
