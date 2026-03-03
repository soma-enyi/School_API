use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Mentor {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub id: Uuid,
    
    #[schema(example = "660e8400-e29b-41d4-a716-446655440001", format = "uuid")]
    pub user_id: Uuid,
    
    #[schema(example = "770e8400-e29b-41d4-a716-446655440002", format = "uuid")]
    pub school_id: Uuid,
    
    #[schema(example = "Mathematics")]
    pub specialization: String,
}
