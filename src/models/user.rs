use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Student {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub school_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStudent {
    pub name: String,
    pub email: String,
    pub school_id: Option<i32>,
}
