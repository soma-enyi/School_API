use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Attendance {
    pub id: Uuid,
    pub course_id: Uuid,
    pub student_id: Uuid,
    pub date: NaiveDate,
    pub status: bool,
    pub marked_by: Uuid,
    pub created_at: DateTime<Utc>,
}
