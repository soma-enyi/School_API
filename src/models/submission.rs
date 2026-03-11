use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AssignmentSubmission {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub student_id: Uuid,
    pub file_url: String,
    pub submitted_at: DateTime<Utc>,
}
