use lettre::message::MultiPart;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::SmtpTransport;
use lettre::{Message, Transport};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use tracing::{error, info};
use utoipa::ToSchema;

use crate::utils::AuthError;

/// Email configuration for SMTP server
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "smtp_host": "smtp.gmail.com",
    "smtp_port": 587,
    "smtp_username": "noreply@school.com",
    "smtp_password": "********",
    "from_email": "noreply@school.com",
    "from_name": "School API"
}))]
pub struct EmailConfig {
    #[schema(example = "smtp.gmail.com")]
    pub smtp_host: String,
    
    #[schema(example = 587)]
    pub smtp_port: u16,
    
    #[schema(example = "noreply@school.com", format = "email")]
    pub smtp_username: String,
    
    #[schema(example = "********", write_only)]
    pub smtp_password: String,
    
    #[schema(example = "noreply@school.com", format = "email")]
    pub from_email: String,
    
    #[schema(example = "School API")]
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
            self.config.smtp_username.clone().into(),
            self.config.smtp_password.clone().into(),
        );

        let mailer = SmtpTransport::relay(&self.config.smtp_host)
            .map_err(|e| {
                error!("SMTP relay error: {}", e);
                AuthError::InternalServerError
            })?
            .port(self.config.smtp_port)
            .credentials(creds)
            .build();

        mailer.send(&message).map_err(|e| {
            error!("Failed to send email: {}", e);
            AuthError::InternalServerError
        })?;

        info!("Email sent successfully to {}", to_email);
        Ok(())
    }

    /// Send OTP email
    pub async fn send_otp_email(
        &self,
        to_email: &str,
        otp: &str,
        user_name: &str,
    ) -> Result<(), AuthError> {
        let subject = "Your OTP for School API";
        let html_body = EmailTemplate::otp_email(user_name, otp);
        let text_body = format!(
            "Hello {},\n\nYour OTP is: {}\n\nThis OTP will expire in 10 minutes.\n\nBest regards,\nSchool API Team",
            user_name, otp
        );

        self.send_email(to_email, subject, &html_body, &text_body)
            .await
    }

    /// Send password reset email
    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        reset_token: &str,
        user_name: &str,
    ) -> Result<(), AuthError> {
        let subject = "Password Reset Request";
        let reset_link = format!(
            "{}?token={}",
            env::var("RESET_PASSWORD_URL")
                .unwrap_or_else(|_| "https://school.com/reset-password".to_string()),
            reset_token
        );
        let html_body = EmailTemplate::password_reset_email(user_name, &reset_link);
        let text_body = format!(
            "Hello {},\n\nClick the link below to reset your password:\n{}\n\nThis link will expire in 1 hour.\n\nBest regards,\nSchool API Team",
            user_name, reset_link
        );

        self.send_email(to_email, subject, &html_body, &text_body)
            .await
    }

    /// Send welcome email
    pub async fn send_welcome_email(
        &self,
        to_email: &str,
        user_name: &str,
        role: &str,
    ) -> Result<(), AuthError> {
        let subject = "Welcome to School API";
        let html_body = EmailTemplate::welcome_email(user_name, role);
        let text_body = format!(
            "Hello {},\n\nWelcome to School API! Your account has been created with role: {}.\n\nBest regards,\nSchool API Team",
            user_name, role
        );

        self.send_email(to_email, subject, &html_body, &text_body)
            .await
    }

    /// Send account verification email
    pub async fn send_verification_email(
        &self,
        to_email: &str,
        verification_token: &str,
        user_name: &str,
    ) -> Result<(), AuthError> {
        let subject = "Verify Your Email Address";
        let verify_link = format!(
            "{}?token={}",
            env::var("VERIFY_EMAIL_URL")
                .unwrap_or_else(|_| "https://school.com/verify-email".to_string()),
            verification_token
        );
        let html_body = EmailTemplate::verification_email(user_name, &verify_link);
        let text_body = format!(
            "Hello {},\n\nClick the link below to verify your email:\n{}\n\nThis link will expire in 24 hours.\n\nBest regards,\nSchool API Team",
            user_name, verify_link
        );

        self.send_email(to_email, subject, &html_body, &text_body)
            .await
    }
}

/// Email template generator
pub struct EmailTemplate;

impl EmailTemplate {
    pub fn otp_email(user_name: &str, otp: &str) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: Arial, sans-serif; background-color: #f4f4f4; }}
        .container {{ max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 20px; border-radius: 8px; }}
        .header {{ background-color: #2c3e50; color: white; padding: 20px; text-align: center; border-radius: 8px 8px 0 0; }}
        .content {{ padding: 20px; }}
        .otp-box {{ background-color: #ecf0f1; padding: 15px; text-align: center; border-radius: 5px; margin: 20px 0; }}
        .otp-code {{ font-size: 32px; font-weight: bold; color: #2c3e50; letter-spacing: 5px; }}
        .footer {{ background-color: #ecf0f1; padding: 15px; text-align: center; font-size: 12px; color: #7f8c8d; border-radius: 0 0 8px 8px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>School API</h1>
        </div>
        <div class="content">
            <p>Hello <strong>{}</strong>,</p>
            <p>Your One-Time Password (OTP) for authentication is:</p>
            <div class="otp-box">
                <div class="otp-code">{}</div>
            </div>
            <p><strong>Important:</strong> This OTP will expire in 10 minutes. Do not share this code with anyone.</p>
            <p>If you did not request this OTP, please ignore this email.</p>
        </div>
        <div class="footer">
            <p>&copy; 2026 School API. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
            "#,
            user_name, otp
        )
    }

    pub fn password_reset_email(user_name: &str, reset_link: &str) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: Arial, sans-serif; background-color: #f4f4f4; }}
        .container {{ max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 20px; border-radius: 8px; }}
        .header {{ background-color: #e74c3c; color: white; padding: 20px; text-align: center; border-radius: 8px 8px 0 0; }}
        .content {{ padding: 20px; }}
        .button {{ display: inline-block; background-color: #3498db; color: white; padding: 12px 30px; text-decoration: none; border-radius: 5px; margin: 20px 0; }}
        .footer {{ background-color: #ecf0f1; padding: 15px; text-align: center; font-size: 12px; color: #7f8c8d; border-radius: 0 0 8px 8px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Password Reset Request</h1>
        </div>
        <div class="content">
            <p>Hello <strong>{}</strong>,</p>
            <p>We received a request to reset your password. Click the button below to proceed:</p>
            <a href="{}" class="button">Reset Password</a>
            <p><strong>Important:</strong> This link will expire in 1 hour.</p>
            <p>If you did not request a password reset, please ignore this email and your password will remain unchanged.</p>
        </div>
        <div class="footer">
            <p>&copy; 2026 School API. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
            "#,
            user_name, reset_link
        )
    }

    pub fn welcome_email(user_name: &str, role: &str) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: Arial, sans-serif; background-color: #f4f4f4; }}
        .container {{ max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 20px; border-radius: 8px; }}
        .header {{ background-color: #27ae60; color: white; padding: 20px; text-align: center; border-radius: 8px 8px 0 0; }}
        .content {{ padding: 20px; }}
        .role-badge {{ display: inline-block; background-color: #3498db; color: white; padding: 8px 15px; border-radius: 20px; margin: 10px 0; }}
        .footer {{ background-color: #ecf0f1; padding: 15px; text-align: center; font-size: 12px; color: #7f8c8d; border-radius: 0 0 8px 8px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Welcome to School API</h1>
        </div>
        <div class="content">
            <p>Hello <strong>{}</strong>,</p>
            <p>Your account has been successfully created!</p>
            <p>Your Role: <span class="role-badge">{}</span></p>
            <p>You can now log in to your account and start using School API.</p>
            <p>If you have any questions, please contact our support team.</p>
        </div>
        <div class="footer">
            <p>&copy; 2026 School API. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
            "#,
            user_name, role
        )
    }

    pub fn verification_email(user_name: &str, verify_link: &str) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: Arial, sans-serif; background-color: #f4f4f4; }}
        .container {{ max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 20px; border-radius: 8px; }}
        .header {{ background-color: #9b59b6; color: white; padding: 20px; text-align: center; border-radius: 8px 8px 0 0; }}
        .content {{ padding: 20px; }}
        .button {{ display: inline-block; background-color: #27ae60; color: white; padding: 12px 30px; text-decoration: none; border-radius: 5px; margin: 20px 0; }}
        .footer {{ background-color: #ecf0f1; padding: 15px; text-align: center; font-size: 12px; color: #7f8c8d; border-radius: 0 0 8px 8px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Verify Your Email Address</h1>
        </div>
        <div class="content">
            <p>Hello <strong>{}</strong>,</p>
            <p>Thank you for signing up! Please verify your email address by clicking the button below:</p>
            <a href="{}" class="button">Verify Email</a>
            <p><strong>Important:</strong> This link will expire in 24 hours.</p>
            <p>If you did not create this account, please ignore this email.</p>
        </div>
        <div class="footer">
            <p>&copy; 2026 School API. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
            "#,
            user_name, verify_link
        )
    }
}
