use sqlx::PgPool;
use crate::models::user::{Student, CreateStudent};

pub async fn get_all_students(pool: &PgPool) -> Result<Vec<Student>, sqlx::Error> {
    sqlx::query_as::<_, Student>("SELECT id, name, email, school_id FROM students")
        .fetch_all(pool)
        .await
}

pub async fn get_student_by_id(pool: &PgPool, id: i32) -> Result<Student, sqlx::Error> {
    sqlx::query_as::<_, Student>("SELECT id, name, email, school_id FROM students WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn create_student(pool: &PgPool, student: CreateStudent) -> Result<Student, sqlx::Error> {
    sqlx::query_as::<_, Student>(
        "INSERT INTO students (name, email, school_id) VALUES ($1, $2, $3) RETURNING id, name, email, school_id"
    )
    .bind(student.name)
    .bind(student.email)
    .bind(student.school_id)
    .fetch_one(pool)
    .await
}
