use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// Database row for an application
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Application {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub course_id: Uuid,
    pub motivation: String,
    pub experience: Option<String>,
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Public form: what an applicant submits
#[derive(Debug, Deserialize, ToSchema)]
pub struct ApplicationRequest {
    /// Applicant's first name
    #[schema(example = "Jane")]
    pub first_name: String,

    /// Applicant's last name
    #[schema(example = "Doe")]
    pub last_name: String,

    /// Applicant's email
    #[schema(example = "jane.doe@example.com")]
    pub email: String,

    /// UUID of the course they are applying for
    pub course_id: Uuid,

    /// Why they chose this course, what motivates them
    #[schema(example = "I want to learn Solidity because I'm passionate about blockchain and decentralized applications.")]
    pub motivation: String,

    /// Prior experience / background (optional)
    #[schema(example = "2 years of JavaScript, basic understanding of smart contracts")]
    pub experience: Option<String>,
}

/// API response for an application
#[derive(Debug, Serialize, ToSchema)]
pub struct ApplicationResponse {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub course_id: Uuid,
    /// Name of the course applied for
    pub course_name: Option<String>,
    pub motivation: String,
    pub experience: Option<String>,
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<Application> for ApplicationResponse {
    fn from(a: Application) -> Self {
        ApplicationResponse {
            id: a.id,
            first_name: a.first_name,
            last_name: a.last_name,
            email: a.email,
            course_id: a.course_id,
            course_name: None, // filled in by service when joining with courses table
            motivation: a.motivation,
            experience: a.experience,
            status: a.status,
            reviewed_by: a.reviewed_by,
            reviewed_at: a.reviewed_at,
            created_at: a.created_at,
        }
    }
}
