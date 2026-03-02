use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct School {
    pub id: i32,
    pub name: String,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSchool {
    pub name: String,
    pub address: Option<String>,
}
