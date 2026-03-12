use lettre::message::MultiPart;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::SmtpTransport;
use lettre::{Message, Transport};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{error, info};

use crate::utils::AuthError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
    pub from_name: String,
}

impl EmailConfig {
    pub fn from_env() -> Result<Self, AuthError> {
        Ok(EmailConfig {
            smtp_host: env::var("SMTP_HOST")
                .unwrap_or_else(|_| "smtp.gmail.com".to_string()),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .unwrap_or(587),
            smtp_username: env::var("SMTP_USERNAME")
                .map_err(|_| AuthError::InternalServerError)?,
            smtp_password: env::var("SMTP_PASSWORD")
                .map_err(|_| AuthError::InternalServerError)?,
            from_email: env::var("FROM_EMAIL")
                .map_err(|_| AuthError::InternalServerError)?,
            from_name: env::var("FROM_NAME")
                .unwrap_or_else(|_| "School API".to_string()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct EmailService {
    pub config: EmailConfig,
}

impl EmailService {
    pub fn new(config: EmailConfig) -> Self {
        EmailService { config }
    }

    /// Send email with HTML and plain text content
    pub async fn send_email(
        &self,
        to_email: &str,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> Result<(), AuthError> {
        let from = format!("{} <{}>", self.config.from_name, self.config.from_email)
            .parse()
            .map_err(|_| AuthError::InternalServerError)?;

        let to = to_email
            .parse()
            .map_err(|_| AuthError::InternalServerError)?;

        let message = Message::builder()
            .from(from)
            .to(to)
            .subject(subject)
            .multipart(MultiPart::alternative()
                .singlepart(lettre::message::SinglePart::plain(text_body.to_string()))
                .singlepart(lettre::message::SinglePart::html(html_body.to_string())))
            .map_err(|_| AuthError::InternalServerError)?;

        let creds = Credentials::new(
            self.config.smtp_username.clone(),
            self.config.smtp_password.clone(),
        );

        // Use STARTTLS for port 587 (Mailtrap and most SMTP services)
        let tls_params = TlsParameters::new(self.config.smtp_host.clone())
            .map_err(|e| {
                error!("TLS parameters error: {}", e);
                AuthError::InternalServerError
            })?;

        let mailer = SmtpTransport::starttls_relay(&self.config.smtp_host)
            .map_err(|e| {
                error!("SMTP relay error: {}", e);
                AuthError::InternalServerError
            })?
            .port(self.config.smtp_port)
            .credentials(creds)
            .tls(Tls::Required(tls_params))
            .build();

        mailer.send(&message).map_err(|e| {
            error!("Failed to send email: {}", e);
            AuthError::InternalServerError
        })?;

        info!("Email sent successfully to {}", to_email);
        Ok(())
    }

    /// Send welcome email
    pub async fn send_welcome_email(
        &self,
        to_email: &str,
        user_name: &str,
        role: &str,
    ) -> Result<(), AuthError> {
        let subject = "Welcome to BuidlFlow!";
        let html_body = EmailTemplate::welcome_email(user_name, role);
        let text_body = format!(
            "Hello {},\n\nWelcome to BuidlFlow! Your account has been created with role: {}.\n\nBest regards,\nCourseFlow Team",
            user_name, role
        );

        self.send_email(to_email, subject, &html_body, &text_body)
            .await
    }

    // ─── Application Pipeline Emails ───

    /// Interview invitation email
    pub async fn send_interview_invitation(
        &self,
        to_email: &str,
        applicant_name: &str,
        course_name: &str,
        interview_venue: &str,
    ) -> Result<(), AuthError> {
        let subject = format!("BuidlFlow: Interview Invitation for {}", course_name);
        let html_body = EmailTemplate::interview_invitation(applicant_name, course_name, interview_venue);
        let text_body = format!(
            "Hello {},\n\nCongratulations! Your application for {} has been reviewed and we would like to invite you for an interview.\n\nInterview Venue: {}\n\nPlease check your email for further scheduling details.\n\nBest regards,\nCourseFlow Admissions Team",
            applicant_name, course_name, interview_venue
        );

        self.send_email(to_email, &subject, &html_body, &text_body).await
    }

    /// Waitlist notification email
    pub async fn send_waitlist_notification(
        &self,
        to_email: &str,
        applicant_name: &str,
        course_name: &str,
    ) -> Result<(), AuthError> {
        let subject = format!("BuidlFlow: You're on the Waitlist for {}", course_name);
        let html_body = EmailTemplate::waitlist_notification(applicant_name, course_name);
        let text_body = format!(
            "Hello {},\n\nGreat news! After your interview, you have been added to the waitlist for {}.\n\nWe will notify you as soon as a spot becomes available.\n\nBest regards,\nCourseFlow Team",
            applicant_name, course_name
        );

        self.send_email(to_email, &subject, &html_body, &text_body).await
    }

    /// Enrollment acceptance email (with temp password)
    pub async fn send_enrollment_acceptance(
        &self,
        to_email: &str,
        applicant_name: &str,
        course_name: &str,
        temp_password: &str,
    ) -> Result<(), AuthError> {
        let subject = format!("BuidlFlow: Welcome to {}!", course_name);
        let login_url = env::var("LOGIN_URL")
            .unwrap_or_else(|_| "https://courseflow.com/login".to_string());
        let html_body = EmailTemplate::enrollment_acceptance(applicant_name, course_name, temp_password, &login_url);
        let text_body = format!(
            "Hello {},\n\nCongratulations! You have been accepted and enrolled in {}.\n\nYour account has been created:\n  Email: {}\n  Temporary Password: {}\n\nPlease log in at {} and change your password immediately.\n\nBest regards,\nCourseFlow Team",
            applicant_name, course_name, to_email, temp_password, login_url
        );

        self.send_email(to_email, &subject, &html_body, &text_body).await
    }

    /// Rejection email
    pub async fn send_rejection_email(
        &self,
        to_email: &str,
        applicant_name: &str,
        course_name: &str,
    ) -> Result<(), AuthError> {
        let subject = format!("BuidlFlow: Update on Your {} Application", course_name);
        let html_body = EmailTemplate::rejection_email(applicant_name, course_name);
        let text_body = format!(
            "Hello {},\n\nThank you for your interest in {}. After careful consideration, we regret to inform you that we are unable to offer you a place in this cohort.\n\nWe encourage you to apply again in the future.\n\nBest regards,\nCourseFlow Team",
            applicant_name, course_name
        );

        self.send_email(to_email, &subject, &html_body, &text_body).await
    }
}

/// Email template generator
pub struct EmailTemplate;


impl EmailTemplate {
    pub fn welcome_email(user_name: &str, role: &str) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: 'Courier New', Courier, monospace; background-color: #ffffff; margin: 0; padding: 40px 20px; color: #1a1a1a; }}
        .container {{ max-width: 560px; margin: 0 auto; }}
        .header {{ border: 2px solid #1a1a1a; padding: 30px; margin-bottom: 30px; }}
        .header .label {{ font-size: 11px; text-transform: uppercase; letter-spacing: 2px; color: #666; margin-bottom: 8px; }}
        .header h1 {{ margin: 0; font-size: 26px; font-weight: bold; }}
        .header p {{ margin: 10px 0 0; font-size: 14px; color: #444; }}
        .content {{ padding: 0 4px; font-size: 14px; line-height: 1.8; }}
        .info-box {{ border: 1px solid #1a1a1a; padding: 20px; margin: 20px 0; }}
        .info-box .label {{ font-size: 11px; text-transform: uppercase; letter-spacing: 2px; color: #666; margin-bottom: 6px; }}
        .info-box .value {{ font-size: 15px; font-weight: bold; }}
        .footer {{ margin-top: 40px; padding-top: 20px; border-top: 1px solid #ddd; font-size: 11px; color: #999; text-transform: uppercase; letter-spacing: 1px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="label">Welcome</div>
            <h1>Welcome to BuidlFlow</h1>
            <p>Your account has been successfully created.</p>
        </div>
        <div class="content">
            <p>Hello <strong>{}</strong>,</p>
            <div class="info-box">
                <div class="label">Your Role</div>
                <div class="value">{}</div>
            </div>
            <p>You can now log in to your account and start using BuidlFlow.</p>
            <p>If you have any questions, please contact our support team.</p>
            <p>Best regards,<br><strong>BuidlFlow Team</strong></p>
        </div>
        <div class="footer">
            &copy; 2026 BuidlFlow. All rights reserved.
        </div>
    </div>
</body>
</html>
            "#,
            user_name, role
        )
    }

    // ─── Application Pipeline Templates ───

    pub fn interview_invitation(applicant_name: &str, course_name: &str, interview_venue: &str) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: 'Courier New', Courier, monospace; background-color: #ffffff; margin: 0; padding: 40px 20px; color: #1a1a1a; }}
        .container {{ max-width: 560px; margin: 0 auto; }}
        .header {{ border: 2px solid #1a1a1a; padding: 30px; margin-bottom: 30px; }}
        .header .label {{ font-size: 11px; text-transform: uppercase; letter-spacing: 2px; color: #666; margin-bottom: 8px; }}
        .header h1 {{ margin: 0; font-size: 26px; font-weight: bold; }}
        .header p {{ margin: 10px 0 0; font-size: 14px; color: #444; }}
        .content {{ padding: 0 4px; font-size: 14px; line-height: 1.8; }}
        .info-box {{ border: 1px solid #1a1a1a; padding: 20px; margin: 20px 0; }}
        .info-box .label {{ font-size: 11px; text-transform: uppercase; letter-spacing: 2px; color: #666; margin-bottom: 6px; }}
        .info-box .value {{ font-size: 15px; font-weight: bold; }}
        .note-box {{ border: 1px solid #ccc; padding: 20px; margin: 20px 0; background-color: #fafafa; }}
        .note-box .label {{ font-size: 11px; text-transform: uppercase; letter-spacing: 2px; color: #666; margin-bottom: 6px; }}
        .footer {{ margin-top: 40px; padding-top: 20px; border-top: 1px solid #ddd; font-size: 11px; color: #999; text-transform: uppercase; letter-spacing: 1px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="label">Interview Invitation</div>
            <h1>Congratulations!</h1>
            <p>You've been invited for an interview.</p>
        </div>
        <div class="content">
            <p>Hello <strong>{}</strong>,</p>
            <p>We are pleased to inform you that your application for <strong>{}</strong> has been reviewed and accepted. We would like to invite you for an interview.</p>
            <div class="info-box">
                <div class="label">Interview Venue</div>
                <div class="value">{}</div>
            </div>

            <p>We look forward to meeting you.</p>
            <p>Best regards,<br><strong>BuidlFlow Admissions Team</strong></p>
        </div>
        <div class="footer">
            &copy; 2026 BuidlFlow. All rights reserved.
        </div>
    </div>
</body>
</html>
            "#,
            applicant_name, course_name, interview_venue
        )
    }

    pub fn waitlist_notification(applicant_name: &str, course_name: &str) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: 'Courier New', Courier, monospace; background-color: #ffffff; margin: 0; padding: 40px 20px; color: #1a1a1a; }}
        .container {{ max-width: 560px; margin: 0 auto; }}
        .header {{ border: 2px solid #1a1a1a; padding: 30px; margin-bottom: 30px; }}
        .header .label {{ font-size: 11px; text-transform: uppercase; letter-spacing: 2px; color: #666; margin-bottom: 8px; }}
        .header h1 {{ margin: 0; font-size: 26px; font-weight: bold; }}
        .header p {{ margin: 10px 0 0; font-size: 14px; color: #444; }}
        .content {{ padding: 0 4px; font-size: 14px; line-height: 1.8; }}
        .note-box {{ border: 1px solid #ccc; padding: 20px; margin: 20px 0; background-color: #fafafa; }}
        .note-box .label {{ font-size: 11px; text-transform: uppercase; letter-spacing: 2px; color: #666; margin-bottom: 6px; }}
        .footer {{ margin-top: 40px; padding-top: 20px; border-top: 1px solid #ddd; font-size: 11px; color: #999; text-transform: uppercase; letter-spacing: 1px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="label">Application Update</div>
            <h1>You're on the Waitlist</h1>
            <p>Great news — you've passed the interview stage.</p>
        </div>
        <div class="content">
            <p>Hello <strong>{}</strong>,</p>
            <p>After your interview, you have been added to the waitlist for <strong>{}</strong>.</p>
            <div class="note-box">
                <div class="label">What does this mean?</div>
                <p style="margin: 6px 0 0;">You've passed the interview stage. We will notify you as soon as a spot becomes available in the course.</p>
            </div>
            <p>Thank you for your patience. We appreciate your interest in BuidlFlow.</p>
            <p>Best regards,<br><strong>BuidlFlow Admissions Team</strong></p>
        </div>
        <div class="footer">
            &copy; 2026 BuidlFlow. All rights reserved.
        </div>
    </div>
</body>
</html>
            "#,
            applicant_name, course_name
        )
    }

    pub fn enrollment_acceptance(applicant_name: &str, course_name: &str, temp_password: &str, login_url: &str) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: 'Courier New', Courier, monospace; background-color: #ffffff; margin: 0; padding: 40px 20px; color: #1a1a1a; }}
        .container {{ max-width: 560px; margin: 0 auto; }}
        .header {{ border: 2px solid #1a1a1a; padding: 30px; margin-bottom: 30px; }}
        .header .label {{ font-size: 11px; text-transform: uppercase; letter-spacing: 2px; color: #666; margin-bottom: 8px; }}
        .header h1 {{ margin: 0; font-size: 26px; font-weight: bold; }}
        .header p {{ margin: 10px 0 0; font-size: 14px; color: #444; }}
        .content {{ padding: 0 4px; font-size: 14px; line-height: 1.8; }}
        .info-box {{ border: 1px solid #1a1a1a; padding: 20px; margin: 20px 0; }}
        .info-box .label {{ font-size: 11px; text-transform: uppercase; letter-spacing: 2px; color: #666; margin-bottom: 6px; }}
        .info-box .value {{ font-size: 15px; font-weight: bold; font-family: 'Courier New', Courier, monospace; }}
        .warning {{ font-size: 13px; color: #666; font-style: italic; margin: 15px 0; }}
        .button {{ display: inline-block; border: 2px solid #1a1a1a; color: #1a1a1a; padding: 12px 28px; text-decoration: none; font-weight: bold; font-family: 'Courier New', Courier, monospace; font-size: 13px; text-transform: uppercase; letter-spacing: 1px; margin: 20px 0; }}
        .footer {{ margin-top: 40px; padding-top: 20px; border-top: 1px solid #ddd; font-size: 11px; color: #999; text-transform: uppercase; letter-spacing: 1px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="label">Enrollment Confirmed</div>
            <h1>Welcome to BuidlFlow</h1>
            <p>You've been accepted and enrolled.</p>
        </div>
        <div class="content">
            <p>Hello <strong>{}</strong>,</p>
            <p>Congratulations! You have been accepted and enrolled in <strong>{}</strong>.</p>
            <div class="info-box">
                <div class="label">Your Temporary Password</div>
                <div class="value">{}</div>
            </div>
            <p class="warning">Please change your password immediately after your first login.</p>
            <a href="{}" class="button">Log In Now</a>
            <p>We're excited to have you on board.</p>
            <p>Best regards,<br><strong>BuidlFlow Team</strong></p>
        </div>
        <div class="footer">
            &copy; 2026 BuidlFlow. All rights reserved.
        </div>
    </div>
</body>
</html>
            "#,
            applicant_name, course_name, temp_password, login_url
        )
    }

    pub fn rejection_email(applicant_name: &str, course_name: &str) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: 'Courier New', Courier, monospace; background-color: #ffffff; margin: 0; padding: 40px 20px; color: #1a1a1a; }}
        .container {{ max-width: 560px; margin: 0 auto; }}
        .header {{ border: 2px solid #1a1a1a; padding: 30px; margin-bottom: 30px; }}
        .header .label {{ font-size: 11px; text-transform: uppercase; letter-spacing: 2px; color: #666; margin-bottom: 8px; }}
        .header h1 {{ margin: 0; font-size: 26px; font-weight: bold; }}
        .header p {{ margin: 10px 0 0; font-size: 14px; color: #444; }}
        .content {{ padding: 0 4px; font-size: 14px; line-height: 1.8; }}
        .note-box {{ border: 1px solid #ccc; padding: 20px; margin: 20px 0; background-color: #fafafa; }}
        .note-box .label {{ font-size: 11px; text-transform: uppercase; letter-spacing: 2px; color: #666; margin-bottom: 6px; }}
        .footer {{ margin-top: 40px; padding-top: 20px; border-top: 1px solid #ddd; font-size: 11px; color: #999; text-transform: uppercase; letter-spacing: 1px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="label">Application Update</div>
            <h1>Application Status</h1>
            <p>An update on your application.</p>
        </div>
        <div class="content">
            <p>Hello <strong>{}</strong>,</p>
            <p>Thank you for your interest in <strong>{}</strong> and for taking the time to apply.</p>
            <p>After careful review, we regret to inform you that we are unable to offer you a place in this cohort.</p>
            <div class="note-box">
                <div class="label">Don't give up</div>
                <p style="margin: 6px 0 0;">We encourage you to continue developing your skills and to apply again for future cohorts. Many successful students applied multiple times before being accepted.</p>
            </div>
            <p>We wish you the best in your learning journey.</p>
            <p>Best regards,<br><strong>BuidlFlow Admissions Team</strong></p>
        </div>
        <div class="footer">
            &copy; 2026 BuidlFlow. All rights reserved.
        </div>
    </div>
</body>
</html>
            "#,
            applicant_name, course_name
        )
    }
}
