use sqlx::PgPool;
use uuid::Uuid;

use crate::models::course::{Course, CreateCourseRequest, UpdateCourseRequest, CourseResponse};
use crate::utils::AuthError;

pub struct CourseService;

impl CourseService {
    /// Create a new course
    pub async fn create_course(
        pool: &PgPool,
        req: CreateCourseRequest,
        created_by: Uuid,
    ) -> Result<CourseResponse, AuthError> {
        let course = sqlx::query_as::<_, Course>(
            "INSERT INTO courses (id, name, description, created_by, created_at)
             VALUES (gen_random_uuid(), $1, $2, $3, NOW())
             RETURNING id, name, description, created_by, created_at"
        )
        .bind(&req.name)
        .bind(&req.description)
        .bind(created_by)
        .fetch_one(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(CourseResponse::from(course))
    }

    /// List all courses
    pub async fn list_courses(pool: &PgPool) -> Result<Vec<CourseResponse>, AuthError> {
        let courses = sqlx::query_as::<_, Course>(
            "SELECT id, name, description, created_by, created_at FROM courses ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(courses.into_iter().map(CourseResponse::from).collect())
    }

    /// Get a single course by ID
    pub async fn get_course(pool: &PgPool, id: Uuid) -> Result<CourseResponse, AuthError> {
        let course = sqlx::query_as::<_, Course>(
            "SELECT id, name, description, created_by, created_at FROM courses WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::UserNotFound)?; // reuse NotFound error

        Ok(CourseResponse::from(course))
    }

    /// Update a course
    pub async fn update_course(
        pool: &PgPool,
        id: Uuid,
        req: UpdateCourseRequest,
    ) -> Result<CourseResponse, AuthError> {
        let course = sqlx::query_as::<_, Course>(
            "UPDATE courses SET
                name = COALESCE($2, name),
                description = COALESCE($3, description)
             WHERE id = $1
             RETURNING id, name, description, created_by, created_at"
        )
        .bind(id)
        .bind(&req.name)
        .bind(&req.description)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::UserNotFound)?;

        Ok(CourseResponse::from(course))
    }

    /// Delete a course
    pub async fn delete_course(pool: &PgPool, id: Uuid) -> Result<(), AuthError> {
        let result = sqlx::query("DELETE FROM courses WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AuthError::UserNotFound);
        }
        Ok(())
    }
}
