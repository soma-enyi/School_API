use sqlx::PgPool;
use std::sync::Arc;

use crate::services::{EmailConfig, EmailService};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub email_service: Option<Arc<EmailService>>,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        // Try to create email service from env vars — if any are missing, use None
        let email_service = EmailConfig::from_env()
            .ok()
            .map(|config| Arc::new(EmailService::new(config)));

        if email_service.is_some() {
            tracing::info!("Email service initialized successfully");
        } else {
            tracing::warn!("Email service not configured — emails will not be sent");
        }

        Self { pool, email_service }
    }

    /// Attempt to send an email, logging but not failing if email service is unavailable
    pub async fn try_send_email<F, Fut>(&self, description: &str, send_fn: F)
    where
        F: FnOnce(Arc<EmailService>) -> Fut,
        Fut: std::future::Future<Output = Result<(), crate::utils::AuthError>>,
    {
        if let Some(ref email_svc) = self.email_service {
            match send_fn(Arc::clone(email_svc)).await {
                Ok(_) => tracing::info!("Email sent: {}", description),
                Err(e) => tracing::error!("Failed to send email ({}): {:?}", description, e),
            }
        } else {
            tracing::warn!("Email not sent (service unavailable): {}", description);
        }
    }
}

// Allow extracting PgPool from AppState directly
impl std::ops::Deref for AppState {
    type Target = PgPool;
    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

impl From<AppState> for PgPool {
    fn from(state: AppState) -> Self {
        state.pool
    }
}
