use sqlx::{FromRow, PgPool};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use crate::services::{EmailService, EmailTemplate};

pub struct SchedulerService {
    pool: PgPool,
    email_service: Option<Arc<EmailService>>,
}

impl SchedulerService {
    pub fn new(pool: PgPool, email_service: Option<Arc<EmailService>>) -> Self {
        Self { pool, email_service }
    }

    pub async fn run(&self) {
        loop {
            if let Err(e) = self.process_registered_applications().await {
                tracing::error!("Scheduler error (registration emails): {:?}", e);
            }
            if let Err(e) = self.process_accepted_applications().await {
                tracing::error!("Scheduler error (acceptance emails): {:?}", e);
            }
            if let Err(e) = self.process_class_reminders().await {
                tracing::error!("Scheduler error (class reminders): {:?}", e);
            }
            if let Err(e) = self.process_dropout_alerts().await {
                tracing::error!("Scheduler error (dropout alerts): {:?}", e);
            }

            sleep(Duration::from_secs(60)).await;
        }
    }

    // ─── Registration Emails ───

    async fn process_registered_applications(&self) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let applications = sqlx::query_as::<_, Application>(
            r#"
            SELECT a.id, a.email, a.first_name || ' ' || a.last_name AS name, c.name AS course_name
            FROM applications a
            JOIN courses c ON c.id = a.course_id
            WHERE a.status = 'pending'
              AND a.welcome_email_sent = FALSE
            FOR UPDATE OF a SKIP LOCKED
            LIMIT 50
            "#,
        )
        .fetch_all(&mut *tx)
        .await?;

        for app in applications {
            if let Some(ref svc) = self.email_service {
                let html = EmailTemplate::course_registration_email(&app.name, &app.course_name);
                let text = format!(
                    "Hello {},\n\nWe received your application for {}. We will review it and get back to you soon.\n\nBuidlFlow Team",
                    app.name, app.course_name
                );
                if let Err(e) = svc.send_email(&app.email, "Application Received", &html, &text).await {
                    tracing::error!("Failed to send registration email to {}: {:?}", app.email, e);
                    continue;
                }
            } else {
                // No email service — skip marking so we retry when it's configured
                continue;
            }

            sqlx::query("UPDATE applications SET welcome_email_sent = TRUE WHERE id = $1")
                .bind(app.id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    // ─── Acceptance Emails ───

    async fn process_accepted_applications(&self) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let applications = sqlx::query_as::<_, Application>(
            r#"
            SELECT a.id, a.email, a.first_name || ' ' || a.last_name AS name, c.name AS course_name
            FROM applications a
            JOIN courses c ON c.id = a.course_id
            WHERE a.status = 'accepted'
              AND a.acceptance_email_sent = FALSE
            FOR UPDATE OF a SKIP LOCKED
            LIMIT 50
            "#,
        )
        .fetch_all(&mut *tx)
        .await?;

        for app in applications {
            if let Some(ref svc) = self.email_service {
                let html = EmailTemplate::course_acceptance_email(&app.name, &app.course_name);
                let text = format!(
                    "Hello {},\n\nCongratulations! Your application for {} has been accepted.\n\nBuidlFlow Team",
                    app.name, app.course_name
                );
                if let Err(e) = svc.send_email(&app.email, "You've Been Accepted!", &html, &text).await {
                    tracing::error!("Failed to send acceptance email to {}: {:?}", app.email, e);
                    continue;
                }
            } else {
                continue;
            }

            sqlx::query("UPDATE applications SET acceptance_email_sent = TRUE WHERE id = $1")
                .bind(app.id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    // ─── Class Start Reminders ───

    async fn process_class_reminders(&self) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let applications = sqlx::query_as::<_, ApplicationWithDate>(
            r#"
            SELECT a.id, a.email, a.first_name || ' ' || a.last_name AS name,
                   c.name AS course_name, a.class_start_date
            FROM applications a
            JOIN courses c ON c.id = a.course_id
            WHERE a.status = 'accepted'
              AND a.class_reminder_sent = FALSE
              AND a.class_start_date IS NOT NULL
              AND a.class_start_date <= NOW()
            FOR UPDATE OF a SKIP LOCKED
            LIMIT 50
            "#,
        )
        .fetch_all(&mut *tx)
        .await?;

        for app in applications {
            if let Some(ref svc) = self.email_service {
                let start_str = app.class_start_date.format("%B %d, %Y").to_string();
                let html = EmailTemplate::course_start_reminder_email(&app.name, &app.course_name, &start_str);
                let text = format!(
                    "Hello {},\n\nYour course {} starts today ({}).\n\nBuidlFlow Team",
                    app.name, app.course_name, start_str
                );
                if let Err(e) = svc.send_email(&app.email, "Your Course Starts Today!", &html, &text).await {
                    tracing::error!("Failed to send reminder email to {}: {:?}", app.email, e);
                    continue;
                }
            } else {
                continue;
            }

            sqlx::query("UPDATE applications SET class_reminder_sent = TRUE WHERE id = $1")
                .bind(app.id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }
    // ─── Dropout Alerts ───

    pub async fn process_dropout_alerts(&self) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Students with 3+ absences not yet alerted
        let absence_cases = sqlx::query_as::<_, DropoutAlert>(
            r#"
            SELECT ce.id AS enrollment_id,
                   u.email,
                   u.first_name || ' ' || u.last_name AS name,
                   c.name AS course_name,
                   'Missed 3 or more classes' AS reason
            FROM course_enrollments ce
            JOIN users u ON u.id = ce.user_id
            JOIN courses c ON c.id = ce.course_id
            WHERE ce.role = 'student'
              AND ce.absence_alert_sent = FALSE
              AND (
                  SELECT COUNT(*) FROM attendance a
                  WHERE a.student_id = ce.user_id
                    AND a.course_id = ce.course_id
                    AND a.status = FALSE
              ) >= 3
            FOR UPDATE OF ce SKIP LOCKED
            LIMIT 50
            "#,
        )
        .fetch_all(&mut *tx)
        .await?;

        // Students with 2+ missed assignment submissions not yet alerted
        let submission_cases = sqlx::query_as::<_, DropoutAlert>(
            r#"
            SELECT ce.id AS enrollment_id,
                   u.email,
                   u.first_name || ' ' || u.last_name AS name,
                   c.name AS course_name,
                   'Failed to submit 2 or more assignments' AS reason
            FROM course_enrollments ce
            JOIN users u ON u.id = ce.user_id
            JOIN courses c ON c.id = ce.course_id
            WHERE ce.role = 'student'
              AND ce.missed_submission_alert_sent = FALSE
              AND (
                  SELECT COUNT(*) FROM assignments a
                  WHERE a.course_id = ce.course_id
                    AND a.due_date < NOW()
                    AND NOT EXISTS (
                        SELECT 1 FROM assignment_submissions s
                        WHERE s.assignment_id = a.id
                          AND s.student_id = ce.user_id
                    )
              ) >= 2
            FOR UPDATE OF ce SKIP LOCKED
            LIMIT 50
            "#,
        )
        .fetch_all(&mut *tx)
        .await?;

        for alert in &absence_cases {
            if let Some(ref svc) = self.email_service {
                let html = EmailTemplate::course_dropout_alert(&alert.name, &alert.course_name, &alert.reason);
                let text = format!(
                    "Hello {},\n\nYou have been dropped from {} due to: {}.\n\nPlease contact your administrator if you have questions.\n\nBuidlFlow Team",
                    alert.name, alert.course_name, alert.reason
                );
                if let Err(e) = svc.send_email(&alert.email, "Important: You Have Been Dropped from a Course", &html, &text).await {
                    tracing::error!("Failed to send absence dropout alert to {}: {:?}", alert.email, e);
                    continue;
                }
            } else {
                continue;
            }
            sqlx::query("UPDATE course_enrollments SET absence_alert_sent = TRUE WHERE id = $1")
                .bind(alert.enrollment_id)
                .execute(&mut *tx)
                .await?;
        }

        for alert in &submission_cases {
            if let Some(ref svc) = self.email_service {
                let html = EmailTemplate::course_dropout_alert(&alert.name, &alert.course_name, &alert.reason);
                let text = format!(
                    "Hello {},\n\nYou have been dropped from {} due to: {}.\n\nPlease contact your administrator if you have questions.\n\nBuidlFlow Team",
                    alert.name, alert.course_name, alert.reason
                );
                if let Err(e) = svc.send_email(&alert.email, "Important: You Have Been Dropped from a Course", &html, &text).await {
                    tracing::error!("Failed to send submission dropout alert to {}: {:?}", alert.email, e);
                    continue;
                }
            } else {
                continue;
            }
            sqlx::query("UPDATE course_enrollments SET missed_submission_alert_sent = TRUE WHERE id = $1")
                .bind(alert.enrollment_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}

// ─── Internal query structs ───

#[derive(Debug, FromRow)]
struct DropoutAlert {
    enrollment_id: Uuid,
    email: String,
    name: String,
    course_name: String,
    reason: String,
}

#[derive(Debug, FromRow)]
struct Application {
    id: Uuid,
    email: String,
    name: String,
    course_name: String,
}

#[derive(Debug, FromRow)]
struct ApplicationWithDate {
    id: Uuid,
    email: String,
    name: String,
    course_name: String,
    class_start_date: chrono::DateTime<chrono::Utc>,
}
