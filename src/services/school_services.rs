use sqlx::PgPool;
use crate::models::school::{School, CreateSchool};

pub async fn get_all_schools(pool: &PgPool) -> Result<Vec<School>, sqlx::Error> {
    sqlx::query_as::<_, School>("SELECT id, name, address FROM schools")
        .fetch_all(pool)
        .await
}

pub async fn get_school_by_id(pool: &PgPool, id: i32) -> Result<School, sqlx::Error> {
    sqlx::query_as::<_, School>("SELECT id, name, address FROM schools WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn create_school(pool: &PgPool, school: CreateSchool) -> Result<School, sqlx::Error> {
    sqlx::query_as::<_, School>(
        "INSERT INTO schools (name, address) VALUES ($1, $2) RETURNING id, name, address"
    )
    .bind(school.name)
    .bind(school.address)
    .fetch_one(pool)
    .await
}
