use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct School {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub id: Uuid,
    
    #[schema(example = "Springfield High School")]
    pub name: String,
    
    #[schema(example = "123 Main St, Springfield")]
    pub location: String,
    
    #[schema(example = "Dr. Jane Smith")]
    pub principal: String,
}
