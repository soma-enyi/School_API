use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct CourseEnrollment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub course_id: Uuid,
    pub role: String,
    pub enrolled_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EnrollRequest {
    pub user_id: Uuid,
    pub course_id: Uuid,
    pub role: String,
}
