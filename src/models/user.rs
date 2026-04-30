use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "VARCHAR")]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum UserRole {
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "student")]
    Student,
    #[serde(rename = "mentor")]
    Mentor,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::Student => write!(f, "student"),
            UserRole::Mentor => write!(f, "mentor"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(UserRole::Admin),
            "student" => Ok(UserRole::Student),
            "mentor" => Ok(UserRole::Mentor),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
#[schema(example = json!({
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@school.com",
    "first_name": "John",
    "last_name": "Doe",
    "role": "student",
    "is_active": true,
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z"
}))]
pub struct User {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub id: Uuid,
    
    #[schema(example = "user@school.com", format = "email")]
    pub email: String,
    
    #[serde(skip_serializing)]
    #[schema(write_only)]
    pub password_hash: String,
    
    #[schema(example = "John")]
    pub first_name: String,
    
    #[schema(example = "Doe")]
    pub last_name: String,
    
    #[schema(example = "student")]
    pub role: String,
    
    #[schema(example = true)]
    pub is_active: bool,
    
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub created_at: DateTime<Utc>,
    
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub updated_at: DateTime<Utc>,

    #[schema(example = "pending")]
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@school.com",
    "first_name": "John",
    "last_name": "Doe",
    "role": "student",
    "is_active": true,
    "created_at": "2024-01-15T10:30:00Z"
}))]
pub struct UserResponse {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid")]
    pub id: Uuid,
    
    #[schema(example = "user@school.com", format = "email")]
    pub email: String,
    
    #[schema(example = "John")]
    pub first_name: String,
    
    #[schema(example = "Doe")]
    pub last_name: String,
    
    #[schema(example = "student")]
    pub role: String,
    
    #[schema(example = true)]
    pub is_active: bool,
    
    #[schema(example = "2024-01-15T10:30:00Z", format = "date-time")]
    pub created_at: DateTime<Utc>,

    #[schema(example = "pending")]
    pub status: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            role: user.role,
            is_active: user.is_active,
            created_at: user.created_at,
            status: user.status,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "email": "user@school.com",
    "password": "SecurePass123!",
    "first_name": "John",
    "last_name": "Doe",
    "role": "student"
}))]
pub struct RegisterRequest {
    #[schema(example = "user@school.com", format = "email")]
    pub email: String,
    
    #[schema(example = "SecurePass123!", min_length = 8)]
    pub password: String,
    
    #[schema(example = "John")]
    pub first_name: String,
    
    #[schema(example = "Doe")]
    pub last_name: String,
    
    #[schema(example = "student", pattern = "^(admin|student|mentor)$")]
    pub role: String, // "admin", "student", or "mentor"
}

#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "email": "mentor@school.com",
    "password": "SecurePass123!",
    "first_name": "Jane",
    "last_name": "Smith",
    "course_id": "550e8400-e29b-41d4-a716-446655440001"
}))]
pub struct MentorRegisterRequest {
    #[schema(example = "mentor@school.com", format = "email")]
    pub email: String,
    
    #[schema(example = "SecurePass123!", min_length = 8)]
    pub password: String,
    
    #[schema(example = "Jane")]
    pub first_name: String,
    
    #[schema(example = "Smith")]
    pub last_name: String,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid")]
    pub course_id: Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "email": "user@school.com",
    "password": "SecurePass123!"
}))]
pub struct LoginRequest {
    #[schema(example = "user@school.com", format = "email")]
    pub email: String,
    
    #[schema(example = "SecurePass123!")]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "user": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "email": "user@school.com",
        "first_name": "John",
        "last_name": "Doe",
        "role": "student",
        "is_active": true,
        "created_at": "2024-01-15T10:30:00Z"
    },
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "token_type": "Bearer",
    "expires_in": 3600
}))]
pub struct AuthResponse {
    pub user: UserResponse,
    
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,
    
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,
    
    #[schema(example = "Bearer")]
    pub token_type: String,
    
    #[schema(example = 3600)]
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}))]
pub struct RefreshTokenRequest {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "token_type": "Bearer",
    "expires_in": 3600
}))]
#[allow(dead_code)]
pub struct TokenResponse {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,
    
    #[schema(example = "Bearer")]
    pub token_type: String,
    
    #[schema(example = 3600)]
    pub expires_in: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // user id
    pub email: String,
    pub role: String,
    pub exp: i64,         // expiration time
    pub iat: i64,         // issued at
    pub token_type: String, // "access" or "refresh"
}
