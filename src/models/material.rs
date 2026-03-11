use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Material {
    pub id: Uuid,
    pub course_id: Uuid,
    pub title: String,
    pub file_url: String,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
}
