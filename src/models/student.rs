use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Student {
    pub id: Uuid,
    pub user_id: Uuid,
    pub school_id: Uuid,
    pub enrollment_date: String,
}
