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
        let subject = "Welcome to CourseFlow!";
        let html_body = EmailTemplate::welcome_email(user_name, role);
        let text_body = format!(
            "Hello {},\n\nWelcome to CourseFlow! Your account has been created with role: {}.\n\nBest regards,\nCourseFlow Team",
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
        let subject = format!("CourseFlow: Interview Invitation for {}", course_name);
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
        let subject = format!("CourseFlow: You're on the Waitlist for {}", course_name);
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
        let subject = format!("CourseFlow: Welcome to {}!", course_name);
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
        let subject = format!("CourseFlow: Update on Your {} Application", course_name);
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
        body {{ font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 20px; }}
        .container {{ max-width: 600px; margin: 0 auto; background-color: #ffffff; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }}
        .header {{ background: linear-gradient(135deg, #11998e 0%, #38ef7d 100%); color: white; padding: 30px; text-align: center; }}
        .header h1 {{ margin: 0; font-size: 28px; }}
        .content {{ padding: 30px; }}
        .role-badge {{ display: inline-block; background-color: #11998e; color: white; padding: 8px 16px; border-radius: 20px; font-weight: bold; margin: 10px 0; }}
        .footer {{ background-color: #f8f9fa; padding: 20px; text-align: center; font-size: 12px; color: #6c757d; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Welcome to CourseFlow</h1>
        </div>
        <div class="content">
            <p>Hello <strong>{}</strong>,</p>
            <p>Your account has been successfully created!</p>
            <p>Your Role: <span class="role-badge">{}</span></p>
            <p>You can now log in to your account and start using CourseFlow.</p>
            <p>If you have any questions, please contact our support team.</p>
        </div>
        <div class="footer">
            <p>&copy; 2026 CourseFlow. All rights reserved.</p>
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
        body {{ font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 20px; }}
        .container {{ max-width: 600px; margin: 0 auto; background-color: #ffffff; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }}
        .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; text-align: center; }}
        .header h1 {{ margin: 0; font-size: 28px; }}
        .content {{ padding: 30px; }}
        .course-badge {{ display: inline-block; background-color: #667eea; color: white; padding: 8px 16px; border-radius: 20px; font-weight: bold; margin: 15px 0; }}
        .venue-box {{ background-color: #f0f4ff; padding: 20px; border-radius: 8px; border-left: 4px solid #667eea; margin: 20px 0; }}
        .venue-box .label {{ font-size: 12px; text-transform: uppercase; color: #667eea; font-weight: bold; letter-spacing: 1px; margin-bottom: 8px; }}
        .venue-box .venue {{ font-size: 16px; font-weight: bold; color: #333; }}
        .highlight {{ background-color: #fff8e1; padding: 20px; border-radius: 8px; border-left: 4px solid #ffc107; margin: 20px 0; }}
        .footer {{ background-color: #f8f9fa; padding: 20px; text-align: center; font-size: 12px; color: #6c757d; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Congratulations!</h1>
            <p style="margin: 10px 0 0; font-size: 16px; opacity: 0.9;">You've been invited for an interview</p>
        </div>
        <div class="content">
            <p>Hello <strong>{}</strong>,</p>
            <p>We are thrilled to inform you that your application for <span class="course-badge">{}</span> has been reviewed and accepted! We would love to invite you for an interview.</p>
            <div class="venue-box">
                <div class="label">Interview Venue</div>
                <div class="venue">{}</div>
            </div>
            <div class="highlight">
                <strong>What to expect:</strong><br>
                Our admissions team will reach out to you with the exact date and time. Please keep an eye on your inbox for scheduling details.
            </div>
            <p>We look forward to meeting you!</p>
            <p>Best regards,<br><strong>CourseFlow Admissions Team</strong></p>
        </div>
        <div class="footer">
            <p>&copy; 2026 CourseFlow. All rights reserved.</p>
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
        body {{ font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 20px; }}
        .container {{ max-width: 600px; margin: 0 auto; background-color: #ffffff; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }}
        .header {{ background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%); color: white; padding: 30px; text-align: center; }}
        .header h1 {{ margin: 0; font-size: 28px; }}
        .content {{ padding: 30px; }}
        .course-badge {{ display: inline-block; background-color: #f5576c; color: white; padding: 8px 16px; border-radius: 20px; font-weight: bold; margin: 15px 0; }}
        .status-box {{ background-color: #fff3cd; padding: 20px; border-radius: 8px; border-left: 4px solid #ffc107; margin: 20px 0; }}
        .footer {{ background-color: #f8f9fa; padding: 20px; text-align: center; font-size: 12px; color: #6c757d; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>You're on the Waitlist!</h1>
        </div>
        <div class="content">
            <p>Hello <strong>{}</strong>,</p>
            <p>Great news! After your interview, you have been added to the waitlist for <span class="course-badge">{}</span>.</p>
            <div class="status-box">
                <strong>What does this mean?</strong><br>
                You've passed the interview stage! We will notify you as soon as a spot becomes available in the course.
            </div>
            <p>Thank you for your patience. We appreciate your interest in CourseFlow!</p>
            <p>Best regards,<br><strong>CourseFlow Admissions Team</strong></p>
        </div>
        <div class="footer">
            <p>&copy; 2026 CourseFlow. All rights reserved.</p>
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
        body {{ font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 20px; }}
        .container {{ max-width: 600px; margin: 0 auto; background-color: #ffffff; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }}
        .header {{ background: linear-gradient(135deg, #11998e 0%, #38ef7d 100%); color: white; padding: 30px; text-align: center; }}
        .header h1 {{ margin: 0; font-size: 28px; }}
        .content {{ padding: 30px; }}
        .course-badge {{ display: inline-block; background-color: #11998e; color: white; padding: 8px 16px; border-radius: 20px; font-weight: bold; margin: 15px 0; }}
        .credentials-box {{ background-color: #e8f5e9; padding: 20px; border-radius: 8px; border-left: 4px solid #4caf50; margin: 20px 0; }}
        .credentials-box code {{ background-color: #c8e6c9; padding: 4px 8px; border-radius: 4px; font-family: monospace; font-size: 16px; }}
        .warning {{ color: #d32f2f; font-weight: bold; }}
        .button {{ display: inline-block; background-color: #11998e; color: white; padding: 14px 30px; text-decoration: none; border-radius: 8px; margin: 20px 0; font-weight: bold; }}
        .footer {{ background-color: #f8f9fa; padding: 20px; text-align: center; font-size: 12px; color: #6c757d; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Welcome to CourseFlow!</h1>
        </div>
        <div class="content">
            <p>Hello <strong>{}</strong>,</p>
            <p>Congratulations! You have been accepted and enrolled in <span class="course-badge">{}</span>.</p>
            <div class="credentials-box">
                <strong>Your Account Credentials:</strong><br><br>
                <strong>Temporary Password:</strong> <code>{}</code>
            </div>
            <p class="warning">Please change your password immediately after your first login.</p>
            <a href="{}" class="button">Log In Now</a>
            <p>We're excited to have you on board!</p>
            <p>Best regards,<br><strong>CourseFlow Team</strong></p>
        </div>
        <div class="footer">
            <p>&copy; 2026 CourseFlow. All rights reserved.</p>
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
        body {{ font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 20px; }}
        .container {{ max-width: 600px; margin: 0 auto; background-color: #ffffff; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }}
        .header {{ background: linear-gradient(135deg, #606c88 0%, #3f4c6b 100%); color: white; padding: 30px; text-align: center; }}
        .header h1 {{ margin: 0; font-size: 28px; }}
        .content {{ padding: 30px; }}
        .course-badge {{ display: inline-block; background-color: #606c88; color: white; padding: 8px 16px; border-radius: 20px; font-weight: bold; margin: 15px 0; }}
        .encourage-box {{ background-color: #e3f2fd; padding: 20px; border-radius: 8px; border-left: 4px solid #2196f3; margin: 20px 0; }}
        .footer {{ background-color: #f8f9fa; padding: 20px; text-align: center; font-size: 12px; color: #6c757d; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Application Update</h1>
        </div>
        <div class="content">
            <p>Hello <strong>{}</strong>,</p>
            <p>Thank you for your interest in <span class="course-badge">{}</span> and for taking the time to apply.</p>
            <p>After careful review, we regret to inform you that we are unable to offer you a place in this cohort.</p>
            <div class="encourage-box">
                <strong>Don't give up!</strong><br>
                We encourage you to continue developing your skills and to apply again for future cohorts. Many successful students applied multiple times before being accepted.
            </div>
            <p>We wish you the best in your learning journey!</p>
            <p>Best regards,<br><strong>CourseFlow Admissions Team</strong></p>
        </div>
        <div class="footer">
            <p>&copy; 2026 CourseFlow. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
            "#,
            applicant_name, course_name
        )
    }
}
